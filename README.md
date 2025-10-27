# 🚀 RustyRedis — A Minimal Redis Clone in Rust

RustyRedis is a lightweight Redis- (key-value data-store) server written in **Rust**, designed to cover the **basic Redis commands** such as `SET`, `GET`, `DEL`, `PING`,`FLUSHALL`, and more.  
It’s built for learning, experimentation, and exploring how Redis works under the hood.

---

## ✨ Features

- ✅ In-memory key-value store  
- ✅ Support for basic Redis commands:
  - `PING`
  - `SET <key> <value>`
  - `GET <key>`
  - `DEL <key>`
  - `EXPIRE <key> <value>`
  - `TTL <key>`
  - `KEYS <glob>`
  
- ✅ Simple TCP-based protocol compatible with the Redis CLI
- ✅ Written entirely in safe Rust 🦀
- ✅ Well-structured and easy to extend

---

## 🧠 Why

Redis is an elegant example of a simple yet powerful server.  
This project helps you **understand how Redis works internally**, from handling sockets to parsing commands and managing in-memory data structures.

---

## ⚙️ Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (version 1.70 or newer)
- Cargo (included with Rust)

### Clone the Repository

```bash
git clone https://github.com/abizGitHub/key-value_data-store.git
cd key-value_data-store

## Build and use the Project

cargo build --release
cargo run 
(run on another port: cargo run -- -p 7676)
(local persistence: cargo run -- PERSIST)
redis-cli -p 6379

## Testing
cargo test

## 🪪 License

This project is licensed under the [MIT License](LICENSE) — feel free to use, modify, and distribute it as you wish.
