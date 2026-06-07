# CraftMan (Discord ↔ Minecraft Server Bridge)

CraftMan is a full-stack system that allows you to control and monitor a Minecraft server directly from Discord. CraftMan offers a way to provide access to remotely managing and interacting with a Minecraft server without the need for remotely connecting via SSH or use of a dedicated server panel to do simple tasks. This also gives access to other users being able to start, stop and manage the server without managing ssh permissions or dedicated logins for each user, simply by managing command permissions on Discord

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
 
* **Verification**
  * Agents can be verified and attached to individual Discord servers
 
* **Minecraft Chat**
  * Agents can forward Minecraft server output to Discord, allowing users to see and interact with the Minecraft servers chat while not actively in the server

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


---

## Project Status

CraftMan is currently in an early public release stage.

The core functionality is implemented and usable, and I am actively using it to manage a Minecraft server with friends. Most testing has been manual and based on real usage rather than a complete automated test suite, so bugs and rough edges should be expected.

---

## 📦 Usage Guide

* First, add the bot to the server you plan on connecting your minecraft server to
* To get started, download the craftman-agent binary, enter the directory it's installed in and run ```./craftman-agent``` (alternatively, you can move craftman-agent to your /bin directory, allowing you to run ```craftman-agent``` from anywhere
* Use the arrow keys to navigate the file selection, and select the server jar file that you will run your minecraft server from
* The agent should connect to the server, and a pop up should appear with the verification code
* On Discord, run ```/verify ####``` and enter your code to connect the agent to the discord server.
* Before running the server from Discord, be sure you have agreed to the EULA and have run the Minecraft server at least once afterwards
* Now you can start and stop the Minecraft server with ```/server start``` and ```/server stop```
* Within the Discord channel where you'd like the bot to forward the chat from minecraft, run ```/chat set``` (It is recommended you mute this channel server wide)
* Start and stop the chat stream to and from the Minecraft server with ```/chat start``` and ```/chat stop```
* Send a message to users in the Minecraft server with ```/chat say```
* Run a command in the Minecraft server with ```/chat command```
* View and manage the properties of the Minecraft server with ```/server properties```
* Run ```/monitor``` to build a live monitor that monitors the current status of the minecraft server

---

## 🎯 Project Goals

This project was built as a portfolio piece to demonstrate:

* Real-world async systems in Rust
* WebSocket-based distributed architecture
* Deep understanding of third-party APIs (Discord)
* Ability to extend and contribute to open-source libraries
* Practical problem-solving without unnecessary rewrites

---

---

## 🤝 Contributions

Open to feedback and collaboration!
This project also includes upstream contributions to the Twilight library.

---

## 📄 License

MIT
