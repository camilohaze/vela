# 5. Arquitectura de Actors y Concurrencia

## 5.1 Modelo de Actores

### 5.1.1 Principios Fundamentales

**Características del Modelo de Actores en Vela**:
1. **Aislamiento de memoria**: Cada actor tiene su propio estado privado
2. **Mensajería asíncrona**: Comunicación solo por mensajes
3. **Location transparency**: No importa dónde ejecute el actor (local/remoto)
4. **Supervisión estructurada**: Jerarquía de supervisión para manejo de errores
5. **Concurrencia implícita**: El runtime maneja threads automáticamente

**Vs. Threading tradicional**:
```
Threading tradicional:
- Shared memory + locks/mutex
- Race conditions posibles
- Deadlocks
- Difícil debugging

Actores en Vela:
- No shared memory
- Sin locks explícitos
- No race conditions
- Supervisión de errores
```

---

### 5.1.2 Anatomía de un Actor

```vela
actor Counter {
  // Estado privado (aislado)
  private count: Int = 0;
  private maxCount: Int = 100;
  
  // Lifecycle hooks
  protected fn onStart(): void {
    print("Counter actor started");
  }
  
  protected fn onStop(): void {
    print("Counter actor stopped");
  }
  
  protected fn onError(error: Error): void {
    print("Error in Counter: ${error.message}");
    // Reiniciar estado
    this.count = 0;
  }
  
  // Message handlers
  on Increment() {
    if (this.count < this.maxCount) {
      this.count += 1;
    }
  }
  
  on Decrement() {
    if (this.count > 0) {
      this.count -= 1;
    }
  }
  
  on GetCount(): Int {
    return this.count;
  }
  
  on Reset() {
    this.count = 0;
  }
  
  on SetMax(newMax: Int) {
    this.maxCount = newMax;
  }
  
  // Private methods (solo accesibles dentro del actor)
  private fn validateCount(): Bool {
    return this.count >= 0 && this.count <= this.maxCount;
  }
}
```

---

## 5.2 Implementación del Runtime de Actores

### 5.2.1 Actor System

```rust
struct ActorSystem {
  // Actor registry
  actors: HashMap<ActorId, ActorInstance>,
  
  // Mailboxes
  mailboxes: HashMap<ActorId, Mailbox>,
  
  // Thread pool para ejecutar actors
  executor: ThreadPoolExecutor,
  
  // Supervision tree
  supervisors: HashMap<ActorId, ActorId>,
  
  // Routing
  router: MessageRouter,
  
  // Config
  config: ActorSystemConfig,
}

struct ActorInstance {
  id: ActorId,
  state: Box<dyn Any>,  // Actor's private state
  vtable: ActorVTable,  // Message handlers
  status: ActorStatus,
  supervisor: Option<ActorId>,
}

enum ActorStatus {
  Starting,
  Running,
  Suspended,
  Stopping,
  Stopped,
  Failed(Error),
}

struct ActorVTable {
  onStart: fn(&mut dyn Any),
  onStop: fn(&mut dyn Any),
  onError: fn(&mut dyn Any, Error),
  messageHandlers: HashMap<MessageId, MessageHandler>,
}

type MessageHandler = fn(&mut dyn Any, Message) -> Result<Option<Value>, Error>;
```

### 5.2.2 Mailbox

```rust
struct Mailbox {
  queue: VecDeque<Message>,
  capacity: usize,
  strategy: MailboxStrategy,
}

enum MailboxStrategy {
  Unbounded,
  Bounded { dropStrategy: DropStrategy },
  PriorityQueue,
}

enum DropStrategy {
  DropOldest,
  DropNewest,
  Reject,
}

struct Message {
  id: MessageId,
  sender: Option<ActorId>,
  payload: Box<dyn Any>,
  timestamp: Timestamp,
  priority: u8,
}

impl Mailbox {
  fn enqueue(&mut self, message: Message) -> Result<(), MailboxError> {
    match self.strategy {
      MailboxStrategy::Unbounded => {
        self.queue.push_back(message);
        Ok(())
      },
      
      MailboxStrategy::Bounded { dropStrategy } => {
        if self.queue.len() >= self.capacity {
          match dropStrategy {
            DropStrategy::DropOldest => {
              self.queue.pop_front();
              self.queue.push_back(message);
              Ok(())
            },
            DropStrategy::DropNewest => {
              // Drop incoming message
              Ok(())
            },
            DropStrategy::Reject => {
              Err(MailboxError::Full)
            },
          }
        } else {
          self.queue.push_back(message);
          Ok(())
        }
      },
      
      MailboxStrategy::PriorityQueue => {
        // Insert based on priority
        let index = self.queue.binary_search_by(|m| {
          message.priority.cmp(&m.priority)
        }).unwrap_or_else(|i| i);
        
        self.queue.insert(index, message);
        Ok(())
      },
    }
  }
  
  fn dequeue(&mut self) -> Option<Message> {
    self.queue.pop_front()
  }
}
```

---

### 5.2.3 Message Processing Loop

```rust
impl ActorSystem {
  fn processActor(&mut self, actor_id: ActorId) {
    let actor = self.actors.get_mut(&actor_id).unwrap();
    let mailbox = self.mailboxes.get_mut(&actor_id).unwrap();
    
    // Process one message
    if let Some(message) = mailbox.dequeue() {
      let result = self.handleMessage(actor, message);
      
      match result {
        Ok(response) => {
          // Si hay response y sender, enviar respuesta
          if let (Some(value), Some(sender)) = (response, message.sender) {
            self.send(sender, Message::Response(value));
          }
        },
        
        Err(error) => {
          // Error handling
          self.handleActorError(actor_id, error);
        },
      }
    }
    
    // Reschedule si hay más mensajes
    if !mailbox.queue.is_empty() {
      self.executor.schedule(actor_id);
    }
  }
  
  fn handleMessage(
    &mut self,
    actor: &mut ActorInstance,
    message: Message
  ) -> Result<Option<Value>, Error> {
    let handler = actor.vtable.messageHandlers.get(&message.id)
      .ok_or(Error::UnknownMessage)?;
    
    handler(&mut *actor.state, message)
  }
  
  fn handleActorError(&mut self, actor_id: ActorId, error: Error) {
    let actor = self.actors.get_mut(&actor_id).unwrap();
    
    // Call onError hook
    (actor.vtable.onError)(&mut *actor.state, error.clone());
    
    // Notify supervisor
    if let Some(supervisor_id) = actor.supervisor {
      self.send(supervisor_id, Message::ChildFailed {
        childId: actor_id,
        error: error,
      });
    }
  }
}
```

---

### 5.2.4 Actor Scheduling

```rust
struct ThreadPoolExecutor {
  threads: Vec<JoinHandle<()>>,
  workQueue: Arc<Mutex<VecDeque<ActorId>>>,
  shutdown: Arc<AtomicBool>,
}

impl ThreadPoolExecutor {
  fn new(num_threads: usize) -> Self {
    let work_queue = Arc::new(Mutex::new(VecDeque::new()));
    let shutdown = Arc::new(AtomicBool::new(false));
    
    let threads = (0..num_threads).map(|_| {
      let queue = work_queue.clone();
      let shutdown = shutdown.clone();
      
      thread::spawn(move || {
        while !shutdown.load(Ordering::Relaxed) {
          let actor_id = {
            let mut queue = queue.lock().unwrap();
            queue.pop_front()
          };
          
          if let Some(actor_id) = actor_id {
            ACTOR_SYSTEM.processActor(actor_id);
          } else {
            // Sleep briefly si no hay trabajo
            thread::sleep(Duration::from_millis(1));
          }
        }
      })
    }).collect();
    
    ThreadPoolExecutor {
      threads,
      workQueue: work_queue,
      shutdown,
    }
  }
  
  fn schedule(&self, actor_id: ActorId) {
    let mut queue = self.workQueue.lock().unwrap();
    queue.push_back(actor_id);
  }
}
```

---

## 5.3 Supervision Hierarchy

### 5.3.1 Supervision Strategies

```vela
enum SupervisionStrategy {
  // Reiniciar solo el actor que falló
  OneForOne,
  
  // Reiniciar todos los actores hermanos
  OneForAll,
  
  // Reiniciar el actor y todos sus hijos
  RestForOne,
}

actor Supervisor {
  private strategy: SupervisionStrategy = SupervisionStrategy.OneForOne;
  private children: List<ActorRef> = [];
  private restartCount: Dict<ActorId, Int> = {};
  private maxRestarts: Int = 3;
  private restartWindow: Duration = Duration.seconds(60);
  
  on ChildFailed(childId: ActorId, error: Error) {
    let count = this.restartCount.get(childId) ?? 0;
    
    if (count >= this.maxRestarts) {
      print("Child ${childId} exceeded max restarts, stopping");
      this.stopChild(childId);
      return;
    }
    
    match (this.strategy) {
      SupervisionStrategy.OneForOne => {
        this.restartChild(childId);
      },
      
      SupervisionStrategy.OneForAll => {
        for (child in this.children) {
          this.restartChild(child.id);
        }
      },
      
      SupervisionStrategy.RestForOne => {
        let index = this.children.findIndex((c) => c.id == childId);
        for (i in index..this.children.length) {
          this.restartChild(this.children[i].id);
        }
      }
    }
    
    this.restartCount.set(childId, count + 1);
  }
  
  private fn restartChild(childId: ActorId): void {
    // Stop actor
    ActorSystem.stop(childId);
    
    // Start new instance
    ActorSystem.start(childId);
  }
}
```

### 5.3.2 Ejemplo de Jerarquía

```vela
actor AppSupervisor {
  private databaseActor: ActorRef;
  private apiActor: ActorRef;
  private cacheActor: ActorRef;
  
  fn onStart(): void {
    // Create child actors
    this.databaseActor = ActorSystem.spawn(DatabaseActor, this);
    this.apiActor = ActorSystem.spawn(ApiActor, this);
    this.cacheActor = ActorSystem.spawn(CacheActor, this);
  }
  
  on ChildFailed(childId: ActorId, error: Error) {
    // Custom supervision logic
    if (childId == this.databaseActor.id) {
      print("Critical: Database actor failed!");
      // Restart con backoff exponencial
      this.restartWithBackoff(childId);
    } else {
      // Otros actores: restart simple
      ActorSystem.restart(childId);
    }
  }
}
```

---

## 5.4 Async/Await Integration

### 5.4.1 Async Functions

```vela
async fn fetchUserData(userId: Int): Result<User, Error> {
  let response = await http.get("/api/users/${userId}");
  let user = await response.json<User>();
  return Result.Ok(user);
}

// Uso
let userData = await fetchUserData(123);
match (userData) {
  Result.Ok(user) => print("User: ${user.name}"),
  Result.Err(error) => print("Error: ${error}")
}
```

### 5.4.2 Async en Actors

```vela
actor DataFetcher {
  private cache: Dict<Int, User> = {};
  
  on FetchUser(userId: Int): async Result<User, Error> {
    // Check cache primero
    if (this.cache.has(userId)) {
      return Result.Ok(this.cache.get(userId));
    }
    
    // Fetch from API (async)
    let result = await fetchUserData(userId);
    
    match (result) {
      Result.Ok(user) => {
        this.cache.set(userId, user);
        return Result.Ok(user);
      },
      Result.Err(error) => {
        return Result.Err(error);
      }
    }
  }
}
```

### 5.4.3 Implementación: State Machine Transform

**Async/await se transforma en state machine**:

```vela
// Código original
async fn example(): Int {
  let x = await asyncOp1();
  let y = await asyncOp2(x);
  return x + y;
}

// Transformado a:
fn example(): Future<Int> {
  enum State {
    Start,
    AwaitOp1(Future<Int>),
    AwaitOp2(Int, Future<Int>),
    Done(Int),
  }
  
  let state = State.Start;
  
  return Future::new(move || {
    loop {
      match (state) {
        State.Start => {
          let future = asyncOp1();
          state = State.AwaitOp1(future);
        },
        
        State.AwaitOp1(future) => {
          if (future.isReady()) {
            let x = future.poll();
            let future2 = asyncOp2(x);
            state = State.AwaitOp2(x, future2);
          } else {
            return Poll.Pending;
          }
        },
        
        State.AwaitOp2(x, future) => {
          if (future.isReady()) {
            let y = future.poll();
            state = State.Done(x + y);
          } else {
            return Poll.Pending;
          }
        },
        
        State.Done(result) => {
          return Poll.Ready(result);
        }
      }
    }
  });
}
```

---

## 5.5 Workers (Thread Pool)

### 5.5.1 Worker API

```vela
// Spawn worker para computación pesada
let worker = Worker.spawn(() => {
  // Esta función ejecuta en thread separado
  let result = 0;
  for (i in 0..1000000) {
    result += i;
  }
  return result;
});

// Esperar resultado
let result = await worker.await();
print("Result: ${result}");
```

### 5.5.2 Implementación

```rust
struct Worker {
  handle: JoinHandle<Box<dyn Any>>,
  result: Arc<Mutex<Option<Box<dyn Any>>>>,
  finished: Arc<AtomicBool>,
}

impl Worker {
  fn spawn<T, F>(task: F) -> Worker
  where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
  {
    let result = Arc::new(Mutex::new(None));
    let finished = Arc::new(AtomicBool::new(false));
    
    let result_clone = result.clone();
    let finished_clone = finished.clone();
    
    let handle = thread::spawn(move || {
      let value = task();
      let boxed: Box<dyn Any> = Box::new(value);
      
      *result_clone.lock().unwrap() = Some(boxed);
      finished_clone.store(true, Ordering::Release);
    });
    
    Worker {
      handle,
      result,
      finished,
    }
  }
  
  async fn await<T>(self) -> T
  where
    T: 'static,
  {
    // Poll hasta que esté listo
    while !self.finished.load(Ordering::Acquire) {
      yield_now().await;
    }
    
    let result = self.result.lock().unwrap().take().unwrap();
    *result.downcast::<T>().unwrap()
  }
}
```

---

## 5.6 Channels (Message Passing)

### 5.6.1 API de Channels

```vela
// Create bounded channel
let (sender, receiver) = Channel.create<Int>(capacity: 10);

// Producer
async fn producer() {
  for (i in 0..100) {
    await sender.send(i);
    print("Sent: ${i}");
  }
  sender.close();
}

// Consumer
async fn consumer() {
  while (true) {
    match (await receiver.receive()) {
      Option.Some(value) => print("Received: ${value}"),
      Option.None => break  // Channel closed
    }
  }
}

// Run concurrently
Task.run(producer());
Task.run(consumer());
```

### 5.6.2 Implementación

```rust
struct Channel<T> {
  buffer: Arc<Mutex<VecDeque<T>>>,
  capacity: usize,
  senders: Arc<AtomicUsize>,
  closed: Arc<AtomicBool>,
  notEmpty: Arc<Condvar>,
  notFull: Arc<Condvar>,
}

struct Sender<T> {
  channel: Arc<Channel<T>>,
}

struct Receiver<T> {
  channel: Arc<Channel<T>>,
}

impl<T> Sender<T> {
  async fn send(&self, value: T) -> Result<(), SendError> {
    loop {
      let mut buffer = self.channel.buffer.lock().unwrap();
      
      if self.channel.closed.load(Ordering::Acquire) {
        return Err(SendError::Closed);
      }
      
      if buffer.len() < self.channel.capacity {
        buffer.push_back(value);
        self.channel.notEmpty.notify_one();
        return Ok(());
      }
      
      // Wait for space
      buffer = self.channel.notFull.wait(buffer).unwrap();
    }
  }
  
  fn close(&self) {
    self.channel.closed.store(true, Ordering::Release);
    self.channel.notEmpty.notify_all();
  }
}

impl<T> Receiver<T> {
  async fn receive(&self) -> Option<T> {
    loop {
      let mut buffer = self.channel.buffer.lock().unwrap();
      
      if let Some(value) = buffer.pop_front() {
        self.channel.notFull.notify_one();
        return Some(value);
      }
      
      if self.channel.closed.load(Ordering::Acquire) {
        return None;
      }
      
      // Wait for data
      buffer = self.channel.notEmpty.wait(buffer).unwrap();
    }
  }
}
```

---

## 5.7 Concurrencia Estructurada

### 5.7.1 Task Groups

```vela
async fn processData(): void {
  let taskGroup = TaskGroup.create();
  
  // Spawn multiple tasks
  taskGroup.spawn(async () => {
    await downloadData();
  });
  
  taskGroup.spawn(async () => {
    await processImages();
  });
  
  taskGroup.spawn(async () => {
    await updateDatabase();
  });
  
  // Wait for ALL tasks to complete
  await taskGroup.waitAll();
  
  print("All tasks completed");
}
```

### 5.7.2 Cancellation

```vela
async fn cancellableTask(): Int {
  let cancellationToken = CancellationToken.create();
  
  let task = Task.run(async () => {
    for (i in 0..1000) {
      if (cancellationToken.isCancelled()) {
        throw CancellationError();
      }
      await Task.delay(10);
    }
    return 42;
  });
  
  // Cancelar después de 1 segundo
  Task.delay(1000).then(() => {
    cancellationToken.cancel();
  });
  
  try {
    return await task.await();
  } catch (e: CancellationError) {
    print("Task was cancelled");
    return -1;
  }
}
```

---

## 5.8 Deadlock Prevention

**Estrategias en Vela**:

1. **No locks explícitos**: Actors eliminan necesidad de locks
2. **No shared mutable state**: Solo mensajes
3. **Timeout en await**: Evita esperas infinitas
4. **Detección de ciclos**: En dependencias de actores

```vela
// Timeout automático
async fn fetchWithTimeout(url: String): Result<Response, Error> {
  return await Task.timeout(
    http.get(url),
    Duration.seconds(30)
  );
}
```

---

## 5.9 Backpressure

```vela
actor RateLimitedProcessor {
  private queue: Queue<Task> = Queue();
  private processing: Bool = false;
  private maxConcurrent: Int = 5;
  private current: Int = 0;
  
  on ProcessTask(task: Task) {
    if (this.current >= this.maxConcurrent) {
      // Aplicar backpressure
      this.queue.enqueue(task);
    } else {
      this.executeTask(task);
    }
  }
  
  private async fn executeTask(task: Task): void {
    this.current += 1;
    
    await task.execute();
    
    this.current -= 1;
    
    // Procesar siguiente de la cola
    if (!this.queue.isEmpty()) {
      let next = this.queue.dequeue();
      this.executeTask(next);
    }
  }
}
```

---

**FIN DEL DOCUMENTO: Arquitectura de Actors y Concurrencia**

Este documento especifica completamente el sistema de concurrencia de Vela.
