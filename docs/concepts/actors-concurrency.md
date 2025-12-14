# Actors and Concurrency

Vela's actor model provides a safe, efficient way to handle concurrency and distributed computation. Actors are independent units of computation that communicate through message passing, eliminating shared mutable state and race conditions.

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Creating Actors](#creating-actors)
3. [Message Passing](#message-passing)
4. [Actor Lifecycle](#actor-lifecycle)
5. [Error Handling](#error-handling)
6. [Advanced Patterns](#advanced-patterns)
7. [Performance](#performance)
8. [Best Practices](#best-practices)

---

## Core Concepts

### Actor Model

The actor model treats actors as the fundamental units of computation. Each actor:

- Has its own isolated state
- Communicates only through messages
- Processes messages sequentially
- Can create other actors
- Can send messages to other actors

### Key Benefits

1. **Isolation**: No shared mutable state
2. **Safety**: No race conditions or deadlocks
3. **Scalability**: Easy to distribute across cores/machines
4. **Fault Tolerance**: Actors can fail independently
5. **Composability**: Easy to combine actors

### Actor vs Threads

| Aspect | Actors | Threads |
|--------|--------|---------|
| State | Isolated | Shared (requires locks) |
| Communication | Messages | Shared memory |
| Failure | Isolated | Can crash whole process |
| Scalability | High | Limited by cores |
| Complexity | Low | High (locks, synchronization) |

---

## Creating Actors

### Basic Actor

```vela
actor Counter {
  state count: Number = 0

  fn receive(message: Message) -> void {
    match message {
      Increment(amount) => count = count + amount
      Decrement(amount) => count = count - amount
      GetCount(replyTo) => replyTo ! CurrentCount(count)
      Reset => count = 0
    }
  }
}
```

### Actor with Initialization

```vela
actor DatabaseConnection {
  state connection: Option<Connection> = None

  fn init(config: DBConfig) -> void {
    connection = Some(connectToDatabase(config))
  }

  fn receive(message: Message) -> void {
    match message {
      Query(sql, replyTo) => {
        result = connection.unwrap().execute(sql)
        replyTo ! QueryResult(result)
      }
      Close => {
        connection.unwrap().close()
        stop()  // Stop the actor
      }
    }
  }
}
```

### Actor Factory

```vela
actor ActorFactory {
  fn receive(message: Message) -> void {
    match message {
      CreateCounter(initial, replyTo) => {
        counter = spawn(Counter(initial))
        replyTo ! CounterCreated(counter)
      }
      CreateWorker(replyTo) => {
        worker = spawn(Worker())
        replyTo ! WorkerCreated(worker)
      }
    }
  }
}
```

---

## Message Passing

### Message Types

```vela
// Define message types
enum CounterMessage {
  Increment(amount: Number),
  Decrement(amount: Number),
  GetCount(replyTo: ActorRef),
  Reset
}

enum CounterResponse {
  CurrentCount(value: Number)
}

// Use in actor
actor Counter {
  state count: Number = 0

  fn receive(message: CounterMessage) -> void {
    match message {
      Increment(amount) => count = count + amount
      Decrement(amount) => count = count - amount
      GetCount(replyTo) => replyTo ! CurrentCount(count)
      Reset => count = 0
    }
  }
}
```

### Sending Messages

```vela
// Create actor
counter = spawn(Counter(0))

// Send fire-and-forget message
counter ! Increment(5)

// Send message with reply
counter ! GetCount(self())  // self() gets current actor

// Handle reply
fn receive(message: Message) -> void {
  match message {
    CurrentCount(value) => print("Count is: ${value}")
  }
}
```

### Actor References

```vela
interface ActorRef {
  // Send message (fire-and-forget)
  !(message: Message) -> void

  // Send with timeout
  ask(message: Message, timeout: Duration) -> Future<Message>

  // Get actor path
  path() -> String

  // Check if actor is alive
  isAlive() -> Bool
}
```

---

## Actor Lifecycle

### Actor States

```vela
enum ActorState {
  Starting,    // Being created
  Running,     // Processing messages
  Stopping,    // Graceful shutdown
  Stopped      // Terminated
}
```

### Lifecycle Hooks

```vela
actor LifecycleExample {
  fn preStart() -> void {
    print("Actor starting...")
    // Initialize resources
  }

  fn postStop() -> void {
    print("Actor stopping...")
    // Clean up resources
  }

  fn preRestart(reason: Error) -> void {
    print("Actor restarting due to: ${reason}")
    // Clean up before restart
  }

  fn postRestart(reason: Error) -> void {
    print("Actor restarted after: ${reason}")
    // Reinitialize after restart
  }

  fn receive(message: Message) -> void {
    match message {
      Stop => stop()  // Graceful stop
      Crash => throw Error("Intentional crash")
    }
  }
}
```

### Stopping Actors

```vela
// Graceful stop
actor ! PoisonPill()

// Force stop
actor.stop()

// Stop with timeout
actor.stop(timeout: Duration.seconds(5))
```

### Actor Supervision

```vela
// Supervisor strategy
enum SupervisorStrategy {
  OneForOne,        // Restart only failed actor
  AllForOne,        // Restart all children
  Escalate         // Escalate failure to parent
}

// Supervisor actor
actor Supervisor {
  strategy: SupervisorStrategy = OneForOne

  fn init() -> void {
    // Create child actors
    worker1 = spawn(Worker())
    worker2 = spawn(Worker())
  }

  fn receive(message: Message) -> void {
    match message {
      ChildFailed(child, reason) => {
        match strategy {
          OneForOne => restart(child)
          AllForOne => restartAll()
          Escalate => throw reason
        }
      }
    }
  }
}
```

---

## Error Handling

### Actor Failures

```vela
actor UnreliableActor {
  fn receive(message: Message) -> void {
    match message {
      RiskyOperation => {
        if random() < 0.1 {  // 10% failure rate
          throw Error("Random failure")
        }
        // Success case
      }
    }
  }
}
```

### Error Recovery

```vela
actor ResilientActor {
  state retryCount: Number = 0
  maxRetries: Number = 3

  fn receive(message: Message) -> void {
    match message {
      ProcessData(data) => {
        try {
          processData(data)
          retryCount = 0  // Reset on success
        } catch (error) {
          if retryCount < maxRetries {
            retryCount = retryCount + 1
            // Retry with backoff
            schedule(self(), ProcessData(data), Duration.seconds(retryCount * 2))
          } else {
            // Give up, notify parent
            parent() ! ProcessingFailed(data, error)
          }
        }
      }
    }
  }
}
```

### Circuit Breaker Pattern

```vela
enum CircuitState {
  Closed,     // Normal operation
  Open,       // Failing, reject requests
  HalfOpen    // Testing if service recovered
}

actor CircuitBreaker {
  state state: CircuitState = Closed
  state failureCount: Number = 0
  failureThreshold: Number = 5
  timeout: Duration = Duration.seconds(60)

  fn receive(message: Message) -> void {
    match message {
      Request(operation, replyTo) => {
        match state {
          Closed => {
            // Try operation
            target ! operation
            // Wait for response or timeout
          }
          Open => {
            replyTo ! CircuitOpen()
          }
          HalfOpen => {
            // Allow one request to test
            target ! operation
            state = Closed  // Assume success, will change on failure
          }
        }
      }
      OperationSuccess => {
        failureCount = 0
        state = Closed
      }
      OperationFailed => {
        failureCount = failureCount + 1
        if failureCount >= failureThreshold {
          state = Open
          schedule(self(), TryReset(), timeout)
        }
      }
      TryReset => {
        state = HalfOpen
      }
    }
  }
}
```

---

## Advanced Patterns

### Actor Hierarchies

```vela
// Root supervisor
actor Application {
  fn init() -> void {
    // Create supervisors
    userSupervisor = spawn(UserSupervisor())
    dataSupervisor = spawn(DataSupervisor())
  }
}

// User supervisor
actor UserSupervisor {
  fn init() -> void {
    // Create user actors
    for i in 0..10 {
      spawn(UserActor(i))
    }
  }
}

// User actor
actor UserActor {
  id: Number

  fn init(id: Number) -> void {
    this.id = id
  }

  fn receive(message: Message) -> void {
    match message {
      UserMessage(userId, data) => {
        if userId == this.id {
          processUserData(data)
        }
      }
    }
  }
}
```

### Load Balancing

```vela
actor LoadBalancer {
  state workers: Array<ActorRef> = []
  state nextWorker: Number = 0

  fn init() -> void {
    // Create worker pool
    for i in 0..5 {
      workers = workers + spawn(Worker())
    }
  }

  fn receive(message: Message) -> void {
    match message {
      WorkRequest(data, replyTo) => {
        // Round-robin distribution
        worker = workers[nextWorker]
        nextWorker = (nextWorker + 1) % workers.length

        worker ! ProcessWork(data, replyTo)
      }
    }
  }
}
```

### Event Sourcing

```vela
enum BankEvent {
  AccountOpened(accountId: String, initialBalance: Float),
  Deposit(accountId: String, amount: Float),
  Withdrawal(accountId: String, amount: Float),
  Transfer(from: String, to: String, amount: Float)
}

actor BankAccount {
  state accountId: String
  state balance: Float = 0.0
  state events: Array<BankEvent> = []

  fn init(accountId: String) -> void {
    this.accountId = accountId
  }

  fn receive(message: Message) -> void {
    match message {
      OpenAccount(initialBalance) => {
        event = AccountOpened(accountId, initialBalance)
        applyEvent(event)
      }
      Deposit(amount) => {
        if amount > 0 {
          event = Deposit(accountId, amount)
          applyEvent(event)
        }
      }
      Withdraw(amount) => {
        if amount > 0 && balance >= amount {
          event = Withdrawal(accountId, amount)
          applyEvent(event)
        }
      }
      GetBalance(replyTo) => {
        replyTo ! CurrentBalance(balance)
      }
    }
  }

  fn applyEvent(event: BankEvent) -> void {
    events = events + event

    match event {
      AccountOpened(_, initial) => balance = initial
      Deposit(_, amount) => balance = balance + amount
      Withdrawal(_, amount) => balance = balance - amount
      Transfer(from, _, amount) => {
        if from == accountId {
          balance = balance - amount
        }
      }
    }
  }
}
```

### Saga Pattern

```vela
actor OrderSaga {
  state orderId: String
  state steps: Array<SagaStep> = []
  state completedSteps: Set<String> = Set.empty()

  fn init(orderId: String) -> void {
    this.orderId = orderId
    steps = [
      SagaStep("reserve_inventory", InventoryService()),
      SagaStep("process_payment", PaymentService()),
      SagaStep("ship_order", ShippingService())
    ]
  }

  fn receive(message: Message) -> void {
    match message {
      StartSaga => executeNextStep()
      StepCompleted(stepId) => {
        completedSteps = completedSteps + stepId
        executeNextStep()
      }
      StepFailed(stepId, error) => {
        // Compensate completed steps
        compensateCompletedSteps()
      }
    }
  }

  fn executeNextStep() -> void {
    nextStep = steps.find(step => !completedSteps.contains(step.id))
    match nextStep {
      Some(step) => step.service ! ExecuteStep(orderId, step.id, self())
      None => parent() ! SagaCompleted(orderId)
    }
  }

  fn compensateCompletedSteps() -> void {
    completedSteps.forEach(stepId => {
      step = steps.find(s => s.id == stepId).unwrap()
      step.service ! CompensateStep(orderId, stepId)
    })
  }
}
```

---

## Performance

### Message Throughput

Actors can process thousands of messages per second. Key factors:

- **Message size**: Smaller messages = higher throughput
- **Processing time**: Keep message handlers fast
- **Mailbox size**: Monitor queue length

### Memory Usage

- **Actor state**: Keep minimal state per actor
- **Message passing**: Prefer immutable messages
- **Garbage collection**: Actors help with GC pressure

### Scaling

```vela
// Scale out
actor Router {
  state routees: Array<ActorRef> = []

  fn init() -> void {
    // Create routees based on load
    cpuCount = Runtime.cpuCount()
    for i in 0..cpuCount {
      routees = routees + spawn(Worker())
    }
  }

  fn receive(message: Message) -> void {
    // Route to appropriate worker
    worker = selectWorker(message)
    worker ! message
  }
}

// Scale up
actor WorkerPool {
  state workers: Array<ActorRef> = []
  state activeWorkers: Number = 0

  fn receive(message: Message) -> void {
    match message {
      Work(item) => {
        if activeWorkers < workers.length {
          // Use existing worker
          worker = getIdleWorker()
          worker ! Process(item)
          activeWorkers = activeWorkers + 1
        } else {
          // Scale up
          newWorker = spawn(Worker())
          workers = workers + newWorker
          newWorker ! Process(item)
          activeWorkers = activeWorkers + 1
        }
      }
      WorkCompleted => {
        activeWorkers = activeWorkers - 1
      }
    }
  }
}
```

---

## Best Practices

### 1. Keep Actors Focused

```vela
// ✅ Single responsibility
actor EmailSender {
  fn receive(message: Message) -> void {
    match message {
      SendEmail(to, subject, body) => sendEmail(to, subject, body)
    }
  }
}

// ❌ Multiple responsibilities
actor EmailAndDatabase {
  fn receive(message: Message) -> void {
    match message {
      SendEmail(to, subject, body) => sendEmail(to, subject, body)
      SaveToDB(data) => saveToDatabase(data)
    }
  }
}
```

### 2. Use Immutable Messages

```vela
// ✅ Immutable messages
struct UserData {
  id: Number,
  name: String,
  email: String
}

// ❌ Mutable messages (avoid)
class MutableUserData {
  var id: Number
  var name: String
  var email: String
}
```

### 3. Handle Failures Gracefully

```vela
actor ReliableWorker {
  fn receive(message: Message) -> void {
    match message {
      Process(data) => {
        try {
          result = processData(data)
          sender() ! Success(result)
        } catch (error) {
          sender() ! Failure(error)
        }
      }
    }
  }
}
```

### 4. Use Actor Hierarchies

```vela
// ✅ Hierarchical supervision
actor ApplicationSupervisor {
  fn init() -> void {
    apiSupervisor = spawn(ApiSupervisor())
    dataSupervisor = spawn(DataSupervisor())
  }
}

actor ApiSupervisor {
  fn init() -> void {
    for i in 0..3 {
      spawn(ApiWorker())
    }
  }
}
```

### 5. Monitor Actor Health

```vela
actor HealthMonitor {
  state actors: Map<String, ActorRef> = Map.empty()

  fn receive(message: Message) -> void {
    match message {
      RegisterActor(name, actor) => {
        actors = actors + (name, actor)
        scheduleHealthCheck(name)
      }
      HealthCheck(name) => {
        actor = actors.get(name)
        match actor {
          Some(ref) => {
            if ref.isAlive() {
              scheduleHealthCheck(name)
            } else {
              handleActorDeath(name)
            }
          }
          None => {}  // Actor removed
        }
      }
    }
  }

  fn scheduleHealthCheck(name: String) -> void {
    schedule(self(), HealthCheck(name), Duration.seconds(30))
  }
}
```

### 6. Avoid Blocking Operations

```vela
// ✅ Non-blocking
actor AsyncWorker {
  fn receive(message: Message) -> void {
    match message {
      ProcessFile(path) => {
        async readFile(path).then(content => {
          result = processContent(content)
          sender() ! ProcessingComplete(result)
        })
      }
    }
  }
}

// ❌ Blocking (bad!)
actor BlockingWorker {
  fn receive(message: Message) -> void {
    match message {
      ProcessFile(path) => {
        content = readFileSync(path)  // Blocks actor!
        result = processContent(content)
        sender() ! ProcessingComplete(result)
      }
    }
  }
}
```

### 7. Use Circuit Breakers

```vela
actor ServiceClient {
  state circuitBreaker: ActorRef

  fn init() -> void {
    circuitBreaker = spawn(CircuitBreaker(targetService))
  }

  fn receive(message: Message) -> void {
    match message {
      MakeRequest(data) => {
        circuitBreaker ! Request(data, self())
      }
      Response(data) => {
        // Handle successful response
      }
      CircuitOpen => {
        // Handle circuit open
        scheduleRetry(data, Duration.seconds(5))
      }
    }
  }
}
```

### 8. Test Actor Interactions

```vela
@test
fn testCounterActor() -> void {
  counter = spawn(Counter(0))

  // Test increment
  counter ! Increment(5)
  counter.ask(GetCount(), Duration.seconds(1)).then(response => {
    assert(response == CurrentCount(5))
  })

  // Test decrement
  counter ! Decrement(2)
  counter.ask(GetCount(), Duration.seconds(1)).then(response => {
    assert(response == CurrentCount(3))
  })
}

@test
fn testActorFailure() -> void {
  supervisor = spawn(TestSupervisor())
  failingActor = spawn(FailingActor())

  supervisor ! Monitor(failingActor)

  failingActor ! MakeItFail()

  // Supervisor should handle failure
  supervisor.ask(GetFailureHandled(), Duration.seconds(1)).then(response => {
    assert(response == Yes)
  })
}
```

---

## Common Patterns

### Request-Response

```vela
actor Calculator {
  fn receive(message: Message) -> void {
    match message {
      Add(a, b, replyTo) => replyTo ! Result(a + b)
      Multiply(a, b, replyTo) => replyTo ! Result(a * b)
    }
  }
}

// Usage
calculator = spawn(Calculator())

// Synchronous style (blocking)
result = calculator.ask(Add(2, 3), Duration.seconds(5))
print("2 + 3 = ${result.value}")

// Asynchronous style
calculator ! Add(2, 3, self())

fn receive(message: Message) -> void {
  match message {
    Result(value) => print("Result: ${value}")
  }
}
```

### Publish-Subscribe

```vela
actor Publisher {
  state subscribers: Array<ActorRef> = []

  fn receive(message: Message) -> void {
    match message {
      Subscribe(subscriber) => {
        subscribers = subscribers + subscriber
      }
      Publish(event) => {
        subscribers.forEach(sub => sub ! event)
      }
    }
  }
}

actor Subscriber {
  name: String

  fn init(name: String) -> void {
    this.name = name
  }

  fn receive(message: Message) -> void {
    match message {
      Event(data) => print("${name} received: ${data}")
    }
  }
}
```

### Worker Pool

```vela
actor WorkerPool {
  state workers: Array<ActorRef> = []
  state jobQueue: Array<Job> = []

  fn init(poolSize: Number) -> void {
    for i in 0..poolSize {
      worker = spawn(Worker(self()))
      workers = workers + worker
    }
  }

  fn receive(message: Message) -> void {
    match message {
      SubmitJob(job) => {
        idleWorker = workers.find(w => w.isIdle())
        match idleWorker {
          Some(worker) => worker ! ProcessJob(job)
          None => jobQueue = jobQueue + job
        }
      }
      JobCompleted(worker) => {
        if !jobQueue.isEmpty() {
          nextJob = jobQueue.head()
          jobQueue = jobQueue.tail()
          worker ! ProcessJob(nextJob)
        }
      }
    }
  }
}
```

---

Actors provide a powerful model for building concurrent, fault-tolerant systems. By following these patterns and best practices, you can create scalable applications that handle complexity through composition rather than shared state.