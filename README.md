# CraftMan (Discord ↔ Minecraft Server Bridge)

CraftMan is a full-stack system that allows you to control and monitor a Minecraft server directly from Discord.

It consists of:

* A **local agent** that runs alongside your Minecraft server
* A **central server** that manages connections
* A **Discord bot interface** for user interaction

Together, these components enable real-time server management, configuration, and monitoring — all from within Discord.

---

## 🚀 Features

### ✅ Implemented

* **Start / Stop Server**

  * Control your Minecraft server with simple Discord commands

* **Server Properties Panel**

  * View and edit `server.properties` through a user-friendly Discord interface

* **Live Query Monitor**

  * Real-time status updates (players, uptime, etc.)
  * Automatically updates a persistent Discord message

* **Persistent WebSocket Connection**

  * Local agent connects to the central server for real-time communication

---

### 🚧 In Progress

* **Minecraft Chat ↔ Discord Bridge**

  * View in-game chat directly in Discord
  * Send messages/commands from Discord into Minecraft

* **User Authentication / Server Linking**

  * Unique server IDs for secure connections
  * Agent-based authentication flow

---

## 🧠 Architecture Overview

```
[ Discord Bot ]
        │
        ▼
[ Central Server (Axum) ]
        │
        ▼
[ Local Agent (Rust) ] ──▶ [ Minecraft Server ]
```

### Flow

1. The **agent** runs locally and connects to the central server via WebSocket
2. The **central server** maintains active connections
3. The **Discord bot** sends commands to the central server
4. Commands are forwarded to the appropriate agent
5. The agent executes actions on the Minecraft server

---

## 🛠️ Tech Stack

* **Language:** Rust
* **Async Runtime:** Tokio
* **Web Framework:** Axum
* **Discord Libraries:**

  * Serenity (primary connection handling)
  * Twilight (advanced API features + custom extensions)
* **Serialization:** Serde
* **Concurrency & State:**

  * `Arc`, `RwLock`
  * `mpsc` channels
  * Connection caching with `HashMap`

---

## 💡 Notable Engineering Decisions

### Hybrid Discord API Approach

* Started with **Serenity** for ease of use
* Integrated **Twilight** to access newer/unsupported Discord features
* Result: best of both worlds without rewriting existing code

### Extending Discord Libraries

* Implemented missing Discord components manually in Twilight
* Contributed the implementation upstream → now a **Twilight contributor**

### Custom Serenity Patch

* Encountered data loss due to incomplete enum coverage in Serenity
* Forked and modified Serenity to preserve raw JSON payloads
* Reconstructed missing structures using Twilight deserialization

### Scalable Connection Management

* Active agents stored in a concurrent `HashMap`
* Efficient communication via async channels (`mpsc`)
* Designed for multiple servers and real-time updates

---

## 🧩 Challenges & Lessons Learned

### Integrating Multiple Discord Libraries

I initially built the project using Serenity, but ran into limitations when newer Discord API features were not yet supported. Instead of rewriting the project, I integrated Twilight alongside Serenity:

* Serenity handles connection management
* Twilight is used for advanced and newer API features

This required a deeper understanding of both libraries and how to bridge them effectively without duplicating logic.

---

### Working with Incomplete API Implementations

While building Discord components, I encountered features that were not implemented in either Serenity or Twilight.

To resolve this:

* I studied how Twilight models Discord components internally
* Implemented the missing component myself
* Contributed the implementation upstream to the Twilight repository

This experience improved my ability to work with large codebases and contribute to open source projects.

---

### Debugging Data Loss in Deserialization

I discovered that certain Discord interaction data was being lost due to incomplete enum handling in Serenity.

Instead of restructuring the entire application:

* I forked Serenity and modified its deserialization logic
* Preserved the raw JSON payload alongside parsed data
* Reconstructed the missing structures using Twilight

This reinforced the importance of understanding serialization boundaries and how abstractions can fail in edge cases.

---

### Designing Concurrent Systems in Rust

Managing multiple active agents and real-time communication required careful handling of shared state and concurrency.

Key patterns used:

* `Arc` and `RwLock` for shared state
* `mpsc` channels for message passing
* Connection caching using a `HashMap`

This project significantly improved my understanding of async Rust, ownership, and safe concurrency patterns.

---

### Avoiding Large Rewrites Through Incremental Design

A recurring theme throughout this project was avoiding unnecessary rewrites.

Instead of restarting when hitting limitations, I:

* Extended existing libraries
* Patched dependencies when needed
* Built abstractions that allowed components to evolve independently

This approach mirrors real-world engineering constraints where maintaining momentum and stability is critical.


## 🔐 Planned Authentication Flow

* Each Discord server is assigned a **unique ID**
* Users configure their local agent with this ID
* Agent authenticates with the central server on connection
* Ensures proper routing of commands and isolation between servers

---

## 📦 Getting Started (Planned)

> Setup instructions will be added once authentication and deployment are finalized.

---

## 🎯 Project Goals

This project was built as a portfolio piece to demonstrate:

* Real-world async systems in Rust
* WebSocket-based distributed architecture
* Deep understanding of third-party APIs (Discord)
* Ability to extend and contribute to open-source libraries
* Practical problem-solving without unnecessary rewrites

---

## 📌 Future Improvements

* Full chat synchronization
* Role-based permissions in Discord
* Web dashboard (optional)
* Plugin/mod support for deeper Minecraft integration

---

## 🤝 Contributions

Open to feedback and collaboration!
This project also includes upstream contributions to the Twilight library.

---

## 📄 License

MIT
