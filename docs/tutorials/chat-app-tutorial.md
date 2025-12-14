# Building a Chat App with Vela Actors

This tutorial demonstrates how to build a real-time chat application using Vela's actor model for concurrency. We'll create a multi-room chat with WebSocket simulation, message persistence, and proper error handling.

## Table of Contents

1. [Project Setup](#project-setup)
2. [Architecture Overview](#architecture-overview)
3. [Core Data Types](#core-data-types)
4. [Chat Server Actor](#chat-server-actor)
5. [Room Manager Actor](#room-manager-actor)
6. [User Session Actor](#user-session-actor)
7. [WebSocket Simulator](#websocket-simulator)
8. [UI Components](#ui-components)
9. [Main Application](#main-application)
10. [Testing](#testing)
11. [Complete Code](#complete-code)

---

## Project Setup

Create a new Vela project:

```bash
vela new chat-app
cd chat-app
```

Project structure:

```
chat-app/
├── src/
│   ├── main.vela
│   ├── actors/
│   │   ├── chat_server.vela
│   │   ├── room_manager.vela
│   │   └── user_session.vela
│   ├── components/
│   │   ├── chat_room.vela
│   │   ├── message_list.vela
│   │   └── message_input.vela
│   ├── services/
│   │   └── websocket_simulator.vela
│   └── types/
│       └── chat.vela
├── tests/
│   └── unit/
├── vela.yaml
└── README.md
```

---

## Architecture Overview

Our chat app uses a hierarchical actor system:

```
ChatApp (UI)
├── ChatServer (main coordinator)
│   ├── RoomManager (manages chat rooms)
│   │   ├── RoomActor (per room)
│   │   │   ├── UserSession (per user in room)
│   │   │   └── MessageHistory
│   │   └── RoomActor (another room)
│   └── WebSocketSimulator (simulates real-time connections)
```

This architecture provides:
- **Isolation**: Each room and user session is isolated
- **Scalability**: Easy to distribute across processes/machines
- **Fault Tolerance**: Room failures don't affect others
- **Concurrency**: Multiple conversations happen simultaneously

---

## Core Data Types

Create `src/types/chat.vela`:

```vela
// User information
struct User {
  id: String,
  name: String,
  avatar: Option<String>,
  isOnline: Bool
}

// Chat message
struct Message {
  id: String,
  userId: String,
  userName: String,
  content: String,
  timestamp: DateTime,
  messageType: MessageType
}

// Message types
enum MessageType {
  Text,
  System,
  Join,
  Leave
}

// Chat room
struct Room {
  id: String,
  name: String,
  description: String,
  users: Array<User>,
  messages: Array<Message>,
  createdAt: DateTime
}

// WebSocket message types
enum WSMessage {
  Connect(userId: String),
  Disconnect(userId: String),
  JoinRoom(roomId: String),
  LeaveRoom(roomId: String),
  SendMessage(content: String),
  MessageReceived(message: Message),
  UserJoined(user: User),
  UserLeft(userId: String),
  Error(message: String)
}
```

---

## Chat Server Actor

Create `src/actors/chat_server.vela`:

```vela
actor ChatServer {
  state roomManager: ActorRef
  state webSocketSim: ActorRef
  state connectedUsers: Map<String, ActorRef> = Map.empty()

  fn init() -> void {
    // Create child actors
    roomManager = spawn(RoomManager())
    webSocketSim = spawn(WebSocketSimulator())

    // Connect actors
    webSocketSim ! ConnectToServer(self())
  }

  fn receive(message: WSMessage) -> void {
    match message {
      Connect(userId) => {
        // Create user session
        userSession = spawn(UserSession(userId, self()))
        connectedUsers = connectedUsers + (userId, userSession)

        // Notify WebSocket simulator
        webSocketSim ! UserConnected(userId, userSession)
      }

      Disconnect(userId) => {
        // Clean up user session
        match connectedUsers.get(userId) {
          Some(session) => {
            session ! Disconnect()
            connectedUsers = connectedUsers.remove(userId)
          }
          None => {}
        }
      }

      JoinRoom(roomId) => {
        // Forward to room manager
        roomManager ! JoinRoom(sender(), roomId)
      }

      LeaveRoom(roomId) => {
        // Forward to room manager
        roomManager ! LeaveRoom(sender(), roomId)
      }

      SendMessage(content) => {
        // Forward to room manager
        roomManager ! SendMessage(sender(), content)
      }

      _ => {
        // Forward other messages to appropriate handlers
        forwardToRoomManager(message)
      }
    }
  }

  fn forwardToRoomManager(message: WSMessage) -> void {
    roomManager ! message
  }

  // Public API for UI
  fn getRooms() -> Future<Array<Room>> {
    roomManager.ask(GetRooms())
  }

  fn createRoom(name: String, description: String) -> Future<Room> {
    roomManager.ask(CreateRoom(name, description))
  }
}
```

---

## Room Manager Actor

Create `src/actors/room_manager.vela`:

```vela
actor RoomManager {
  state rooms: Map<String, ActorRef> = Map.empty()
  state roomList: Array<Room> = []

  fn init() -> void {
    // Create default rooms
    createDefaultRooms()
  }

  fn createDefaultRooms() -> void {
    generalRoom = createRoom("general", "General discussion")
    randomRoom = createRoom("random", "Random chat")
    techRoom = createRoom("tech", "Technology discussions")

    roomList = [generalRoom, randomRoom, techRoom]
  }

  fn createRoom(name: String, description: String) -> Room {
    roomId = generateRoomId()
    room = Room {
      id: roomId,
      name: name,
      description: description,
      users: [],
      messages: [],
      createdAt: DateTime.now()
    }

    roomActor = spawn(RoomActor(room))
    rooms = rooms + (roomId, roomActor)

    return room
  }

  fn receive(message: Message) -> void {
    match message {
      GetRooms(replyTo) => {
        replyTo ! RoomsList(roomList)
      }

      CreateRoom(name, description, replyTo) => {
        newRoom = createRoom(name, description)
        roomList = roomList + newRoom
        replyTo ! RoomCreated(newRoom)
      }

      JoinRoom(userSession, roomId) => {
        match rooms.get(roomId) {
          Some(roomActor) => roomActor ! JoinRoom(userSession)
          None => userSession ! Error("Room not found")
        }
      }

      LeaveRoom(userSession, roomId) => {
        match rooms.get(roomId) {
          Some(roomActor) => roomActor ! LeaveRoom(userSession)
          None => userSession ! Error("Room not found")
        }
      }

      SendMessage(userSession, content) => {
        // Find user's current room and forward
        rooms.values().forEach(roomActor => {
          roomActor ! CheckAndForwardMessage(userSession, content)
        })
      }

      // Handle room-specific messages
      RoomUpdated(roomId, updatedRoom) => {
        roomList = roomList.map(room =>
          if room.id == roomId { updatedRoom } else { room }
        )
      }
    }
  }

  fn generateRoomId() -> String {
    return "room_${Date.now()}_${Math.random().toString().substring(2, 8)}"
  }
}
```

---

## Room Actor

Create `src/actors/room_actor.vela`:

```vela
actor RoomActor {
  state room: Room
  state userSessions: Map<String, ActorRef> = Map.empty()
  state messageHistory: Array<Message> = []

  fn init(initialRoom: Room) -> void {
    room = initialRoom
    messageHistory = initialRoom.messages
  }

  fn receive(message: Message) -> void {
    match message {
      JoinRoom(userSession) => {
        // Add user to room
        userId = getUserIdFromSession(userSession)
        userSessions = userSessions + (userId, userSession)

        // Update room state
        user = getUserFromSession(userSession)
        room = Room {
          ...room,
          users: room.users + user
        }

        // Notify all users in room
        broadcast(UserJoined(user))

        // Send welcome message
        systemMessage = Message {
          id: generateMessageId(),
          userId: "system",
          userName: "System",
          content: "${user.name} joined the room",
          timestamp: DateTime.now(),
          messageType: Join
        }

        broadcast(MessageReceived(systemMessage))
        addToHistory(systemMessage)

        // Send room state to new user
        userSession ! RoomJoined(room, messageHistory)
      }

      LeaveRoom(userSession) => {
        userId = getUserIdFromSession(userSession)

        match userSessions.get(userId) {
          Some(_) => {
            userSessions = userSessions.remove(userId)

            // Update room state
            room = Room {
              ...room,
              users: room.users.filter(u => u.id != userId)
            }

            // Notify remaining users
            user = getUserFromSession(userSession)
            broadcast(UserLeft(userId))

            systemMessage = Message {
              id: generateMessageId(),
              userId: "system",
              userName: "System",
              content: "${user.name} left the room",
              timestamp: DateTime.now(),
              messageType: Leave
            }

            broadcast(MessageReceived(systemMessage))
            addToHistory(systemMessage)
          }
          None => {}
        }
      }

      CheckAndForwardMessage(userSession, content) => {
        userId = getUserIdFromSession(userSession)

        if userSessions.containsKey(userId) {
          user = getUserFromSession(userSession)
          newMessage = Message {
            id: generateMessageId(),
            userId: userId,
            userName: user.name,
            content: content,
            timestamp: DateTime.now(),
            messageType: Text
          }

          broadcast(MessageReceived(newMessage))
          addToHistory(newMessage)
        }
      }

      GetRoomState(replyTo) => {
        replyTo ! RoomState(room, messageHistory)
      }
    }
  }

  fn broadcast(message: WSMessage) -> void {
    userSessions.values().forEach(session => {
      session ! message
    })
  }

  fn addToHistory(message: Message) -> void {
    messageHistory = messageHistory + message

    // Keep only last 100 messages
    if messageHistory.length > 100 {
      messageHistory = messageHistory.slice(-100)
    }
  }

  fn getUserIdFromSession(session: ActorRef) -> String {
    // In real implementation, this would extract user ID from session
    return "user_${session.hashCode()}"
  }

  fn getUserFromSession(session: ActorRef) -> User {
    // In real implementation, this would get user data from session
    userId = getUserIdFromSession(session)
    return User {
      id: userId,
      name: "User ${userId.substring(5)}",
      avatar: None,
      isOnline: true
    }
  }

  fn generateMessageId() -> String {
    return "msg_${Date.now()}_${Math.random().toString().substring(2, 8)}"
  }
}
```

---

## User Session Actor

Create `src/actors/user_session.vela`:

```vela
actor UserSession {
  state userId: String
  state chatServer: ActorRef
  state currentRoom: Option<String> = None
  state messageQueue: Array<WSMessage> = []

  fn init(userId: String, chatServer: ActorRef) -> void {
    this.userId = userId
    this.chatServer = chatServer
  }

  fn receive(message: WSMessage) -> void {
    match message {
      // From WebSocket
      JoinRoom(roomId) => {
        // Leave current room if any
        match currentRoom {
          Some(current) => chatServer ! LeaveRoom(current)
          None => {}
        }

        // Join new room
        currentRoom = Some(roomId)
        chatServer ! JoinRoom(roomId)
      }

      LeaveRoom(roomId) => {
        if currentRoom == Some(roomId) {
          currentRoom = None
          chatServer ! LeaveRoom(roomId)
        }
      }

      SendMessage(content) => {
        chatServer ! SendMessage(content)
      }

      // From Chat Server
      RoomJoined(room, history) => {
        // Send room state and history to UI
        sendToUI(RoomJoined(room, history))
      }

      MessageReceived(msg) => {
        sendToUI(MessageReceived(msg))
      }

      UserJoined(user) => {
        sendToUI(UserJoined(user))
      }

      UserLeft(userId) => {
        sendToUI(UserLeft(userId))
      }

      Error(errorMsg) => {
        sendToUI(Error(errorMsg))
      }

      Disconnect() => {
        // Leave current room
        match currentRoom {
          Some(roomId) => {
            chatServer ! LeaveRoom(roomId)
            currentRoom = None
          }
          None => {}
        }

        // Clean up
        stop()
      }
    }
  }

  fn sendToUI(message: WSMessage) -> void {
    // In real implementation, this would send to WebSocket
    // For now, we'll use a global event system
    ChatApp.handleMessage(message)
  }
}
```

---

## WebSocket Simulator

Create `src/services/websocket_simulator.vela`:

```vela
actor WebSocketSimulator {
  state connectedClients: Map<String, ActorRef> = Map.empty()
  state chatServer: Option<ActorRef> = None

  fn receive(message: Message) -> void {
    match message {
      ConnectToServer(server) => {
        chatServer = Some(server)
      }

      UserConnected(userId, session) => {
        connectedClients = connectedClients + (userId, session)
      }

      UserDisconnected(userId) => {
        connectedClients = connectedClients.remove(userId)
      }

      // Simulate incoming WebSocket messages
      SimulateUserMessage(userId, wsMessage) => {
        match connectedClients.get(userId) {
          Some(session) => session ! wsMessage
          None => {}
        }
      }

      // Broadcast to all connected clients
      Broadcast(message) => {
        connectedClients.values().forEach(client => {
          client ! message
        })
      }
    }
  }

  // Public API for testing
  fn simulateUserJoin(userId: String, roomId: String) -> void {
    self() ! SimulateUserMessage(userId, JoinRoom(roomId))
  }

  fn simulateUserMessage(userId: String, content: String) -> void {
    self() ! SimulateUserMessage(userId, SendMessage(content))
  }

  fn simulateUserLeave(userId: String, roomId: String) -> void {
    self() ! SimulateUserMessage(userId, LeaveRoom(roomId))
  }
}
```

---

## UI Components

### ChatRoom Component

Create `src/components/chat_room.vela`:

```vela
component ChatRoom {
  state room: Option<Room> = None
  state messages: Array<Message> = []
  state users: Array<User> = []
  state currentMessage: String = ""

  computed isConnected: Bool {
    return room.isSome()
  }

  fn joinRoom(roomId: String) -> void {
    ChatApp.joinRoom(roomId)
  }

  fn sendMessage() -> void {
    if !currentMessage.trim().isEmpty() {
      ChatApp.sendMessage(currentMessage)
      currentMessage = ""
    }
  }

  fn handleKeyPress(event: KeyEvent) -> void {
    if event.key == "Enter" {
      sendMessage()
    }
  }

  render {
    return Container(
      style: {
        height: "100vh",
        display: "flex",
        flexDirection: "column"
      },
      child: if isConnected {
        Column(
          children: [
            // Room header
            Container(
              style: {
                padding: "10px",
                borderBottom: "1px solid #ddd",
                backgroundColor: "#f5f5f5"
              },
              child: Row(
                children: [
                  Text(
                    match room {
                      Some(r) => r.name
                      None => "No room"
                    },
                    style: { fontSize: "18px", fontWeight: "bold" }
                  ),
                  Spacer(),
                  Text("${users.length} users online")
                ]
              )
            ),

            // Messages area
            Expanded(
              child: MessageList(messages: messages)
            ),

            // Message input
            Container(
              style: {
                padding: "10px",
                borderTop: "1px solid #ddd"
              },
              child: Row(
                children: [
                  Expanded(
                    child: TextInput(
                      value: currentMessage,
                      onChange: (value) => currentMessage = value,
                      onKeyPress: handleKeyPress,
                      placeholder: "Type a message...",
                      style: {
                        padding: "8px",
                        border: "1px solid #ddd",
                        borderRadius: "4px"
                      }
                    )
                  ),
                  Button(
                    "Send",
                    onClick: sendMessage,
                    style: {
                      marginLeft: "10px",
                      padding: "8px 16px",
                      backgroundColor: "#007bff",
                      color: "white",
                      border: "none",
                      borderRadius: "4px",
                      cursor: "pointer"
                    }
                  )
                ]
              )
            )
          ]
        )
      } else {
        // Room selection
        RoomSelector(onRoomSelect: joinRoom)
      }
    )
  }
}
```

### MessageList Component

Create `src/components/message_list.vela`:

```vela
component MessageList {
  state messages: Array<Message>
  state messagesEndRef: Option<Element> = None

  effect {
    // Auto-scroll to bottom when new messages arrive
    match messagesEndRef {
      Some(ref) => ref.scrollIntoView()
      None => {}
    }
  }

  render {
    return Container(
      style: {
        flex: 1,
        overflowY: "auto",
        padding: "10px"
      },
      children: messages.map(message => MessageItem(message: message)) + [
        // Invisible element to scroll to
        Container(
          ref: (el) => messagesEndRef = Some(el),
          style: { height: "1px" }
        )
      ]
    )
  }
}

component MessageItem {
  state message: Message

  computed isSystemMessage: Bool {
    return message.messageType != Text
  }

  computed formattedTime: String {
    return message.timestamp.format("HH:mm")
  }

  render {
    return Container(
      style: {
        marginBottom: "8px",
        padding: "8px",
        borderRadius: "4px",
        backgroundColor: if isSystemMessage { "#f0f0f0" } else { "transparent" }
      },
      child: if isSystemMessage {
        Text(
          message.content,
          style: {
            fontStyle: "italic",
            color: "#666",
            fontSize: "14px",
            textAlign: "center"
          }
        )
      } else {
        Row(
          children: [
            Container(
              style: {
                width: "32px",
                height: "32px",
                borderRadius: "50%",
                backgroundColor: "#007bff",
                color: "white",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                marginRight: "8px",
                fontSize: "14px",
                fontWeight: "bold"
              },
              child: Text(message.userName.substring(0, 1).toUpperCase())
            ),
            Column(
              children: [
                Row(
                  children: [
                    Text(
                      message.userName,
                      style: { fontWeight: "bold", marginRight: "8px" }
                    ),
                    Text(
                      formattedTime,
                      style: { fontSize: "12px", color: "#666" }
                    )
                  ]
                ),
                Text(message.content)
              ]
            )
          ]
        )
      }
    )
  }
}
```

### RoomSelector Component

Create `src/components/room_selector.vela`:

```vela
component RoomSelector {
  state rooms: Array<Room> = []
  state onRoomSelect: (String) -> void

  effect {
    // Load available rooms
    async ChatApp.getRooms().then(rooms => {
      this.rooms = rooms
    })
  }

  render {
    return Container(
      style: {
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        height: "100vh",
        padding: "20px"
      },
      children: [
        Text(
          "Select a Chat Room",
          style: {
            fontSize: "24px",
            fontWeight: "bold",
            marginBottom: "20px"
          }
        ),
        Column(
          children: rooms.map(room => RoomCard(room: room, onSelect: onRoomSelect)),
          spacing: 10
        )
      ]
    )
  }
}

component RoomCard {
  state room: Room
  state onSelect: (String) -> void

  fn handleClick() -> void {
    onSelect(room.id)
  }

  render {
    return Container(
      style: {
        padding: "20px",
        border: "1px solid #ddd",
        borderRadius: "8px",
        cursor: "pointer",
        backgroundColor: "white",
        boxShadow: "0 2px 4px rgba(0,0,0,0.1)",
        transition: "box-shadow 0.2s"
      },
      onClick: handleClick,
      onMouseEnter: () => {
        // Add hover effect
        this.style.boxShadow = "0 4px 8px rgba(0,0,0,0.2)"
      },
      onMouseLeave: () => {
        // Remove hover effect
        this.style.boxShadow = "0 2px 4px rgba(0,0,0,0.1)"
      },
      child: Column(
        children: [
          Text(
            room.name,
            style: {
              fontSize: "18px",
              fontWeight: "bold",
              marginBottom: "8px"
            }
          ),
          Text(
            room.description,
            style: {
              color: "#666",
              marginBottom: "8px"
            }
          ),
          Text(
            "${room.users.length} users online",
            style: {
              fontSize: "14px",
              color: "#007bff"
            }
          )
        ]
      )
    )
  }
}
```

---

## Main Application

Create `src/main.vela`:

```vela
// Global chat application state
store ChatApp {
  state chatServer: Option<ActorRef> = None
  state currentUser: Option<User> = None
  state currentRoom: Option<Room> = None
  state messages: Array<Message> = []
  state users: Array<User> = []
  state availableRooms: Array<Room> = []

  fn init() -> void {
    // Create chat server
    chatServer = Some(spawn(ChatServer()))

    // Load available rooms
    loadRooms()
  }

  fn loadRooms() -> void {
    match chatServer {
      Some(server) => {
        async server.getRooms().then(rooms => {
          availableRooms = rooms
        })
      }
      None => {}
    }
  }

  fn joinRoom(roomId: String) -> void {
    match chatServer {
      Some(server) => {
        server ! JoinRoom(roomId)
      }
      None => {}
    }
  }

  fn sendMessage(content: String) -> void {
    match chatServer {
      Some(server) => {
        server ! SendMessage(content)
      }
      None => {}
    }
  }

  fn handleMessage(message: WSMessage) -> void {
    match message {
      RoomJoined(room, history) => {
        currentRoom = Some(room)
        messages = history
        users = room.users
      }

      MessageReceived(msg) => {
        messages = messages + msg
      }

      UserJoined(user) => {
        users = users + user
      }

      UserLeft(userId) => {
        users = users.filter(u => u.id != userId)
      }

      Error(errorMsg) => {
        // Show error to user
        print("Chat error: ${errorMsg}")
      }

      _ => {}
    }
  }

  fn createRoom(name: String, description: String) -> void {
    match chatServer {
      Some(server) => {
        async server.createRoom(name, description).then(room => {
          availableRooms = availableRooms + room
        })
      }
      None => {}
    }
  }
}

component ChatApplication {
  render {
    return ChatRoom()
  }
}

fn main() {
  ChatApp.init()
  mount(ChatApplication(), document.body)
}
```

---

## Testing

Create `tests/unit/chat_server_test.vela`:

```vela
@test
fn test_chat_server_creation() -> void {
  server = spawn(ChatServer())

  // Server should be created successfully
  assert(server.isAlive())
}

@test
fn test_room_creation() -> void {
  server = spawn(ChatServer())

  // Create a room
  room = await server.createRoom("Test Room", "A test room")

  assert(room.name == "Test Room")
  assert(room.description == "A test room")
  assert(room.users.isEmpty())
}

@test
fn test_user_join_leave() -> void {
  server = spawn(ChatServer())
  room = await server.createRoom("Test Room", "A test room")

  // Simulate user connection
  server ! Connect("user1")

  // Give actors time to process
  await sleep(100)

  // User joins room
  server ! JoinRoom(room.id)

  // Check room state
  roomState = await server.getRoomState(room.id)
  assert(roomState.users.length == 1)
  assert(roomState.users[0].id == "user1")

  // User leaves room
  server ! LeaveRoom(room.id)

  roomState = await server.getRoomState(room.id)
  assert(roomState.users.isEmpty())
}

@test
fn test_message_sending() -> void {
  server = spawn(ChatServer())
  room = await server.createRoom("Test Room", "A test room")

  // Connect and join room
  server ! Connect("user1")
  await sleep(100)
  server ! JoinRoom(room.id)

  // Send message
  server ! SendMessage("Hello, world!")

  // Check messages
  roomState = await server.getRoomState(room.id)
  assert(roomState.messages.length == 1)
  assert(roomState.messages[0].content == "Hello, world!")
  assert(roomState.messages[0].userId == "user1")
}
```

Run tests:

```bash
vela test
```

---

## Complete Code

### src/types/chat.vela
```vela
struct User {
  id: String,
  name: String,
  avatar: Option<String>,
  isOnline: Bool
}

struct Message {
  id: String,
  userId: String,
  userName: String,
  content: String,
  timestamp: DateTime,
  messageType: MessageType
}

enum MessageType {
  Text,
  System,
  Join,
  Leave
}

struct Room {
  id: String,
  name: String,
  description: String,
  users: Array<User>,
  messages: Array<Message>,
  createdAt: DateTime
}
```

### src/actors/chat_server.vela
```vela
actor ChatServer {
  state roomManager: ActorRef

  fn init() -> void {
    roomManager = spawn(RoomManager())
  }

  fn receive(message: WSMessage) -> void {
    match message {
      JoinRoom(roomId) => roomManager ! JoinRoom(sender(), roomId)
      SendMessage(content) => roomManager ! SendMessage(sender(), content)
      _ => {}
    }
  }
}
```

### src/main.vela
```vela
store ChatApp {
  state messages: Array<Message> = []

  fn sendMessage(content: String) -> void {
    newMessage = Message {
      id: "msg_${Date.now()}",
      userId: "user1",
      userName: "You",
      content: content,
      timestamp: DateTime.now(),
      messageType: Text
    }
    messages = messages + newMessage
  }
}

component ChatApp {
  render {
    return Column(
      children: [
        MessageList(messages: ChatApp.messages),
        MessageInput(onSend: ChatApp.sendMessage)
      ]
    )
  }
}

fn main() {
  mount(ChatApp(), document.body)
}
```

---

## Next Steps

Enhance your chat app with:

1. **Real WebSockets**: Replace simulator with actual WebSocket connections
2. **User Authentication**: Add login/signup functionality
3. **Private Messages**: Direct messaging between users
4. **File Sharing**: Upload and share files in chat
5. **Message Reactions**: Add emoji reactions to messages
6. **Typing Indicators**: Show when users are typing
7. **Message History**: Persistent message storage with database
8. **Push Notifications**: Browser notifications for new messages

The complete source code for this tutorial is available in the `examples/chat-app/` directory.