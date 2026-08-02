# LightX API - Demonstration and Architecture Guide

Welcome to the demonstration API of the LightX framework. This project serves as a technological and pedagogical showcase to understand how to interact with the incredible capabilities of the automated code generation engine powered by **LightX** and **Daox**.

[English](README.md) | [Français](README.fr.md)

---

## 🚀 Quick Start (Step-by-Step Tutorial)

This guide is designed to be accessible to everyone, even absolute beginners. Please follow every step carefully.

### Step 1: Prepare your Development Environment

To run this project, you need the foundational Rust programming tools.

- **Install Rust**: If you haven't yet, install the compiler by opening your terminal and running the official command:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  _(⚠️ Crucial for beginners: Once the installation finishes, carefully restart your terminal or execute `source $HOME/.cargo/env` to activate the `cargo` command)._

### Step 2: Prepare the Databases

This demonstration project showcases LightX's power on 3 databases simultaneously. Therefore, the API requires these databases to be reachable before starting.

1. **SQLite**: Runs natively in standard RAM (`sqlite::memory:`). **No installation required!**
2. **PostgreSQL & MySQL**: The server will seamlessly attempt to connect using the credentials present in the `.env` file (`localhost:5432` and `localhost:3306`).

   👉 **The simplest solution (via Docker)**: If you possess Docker, you can instantly spin up and host both databases with the proper credentials using these two simple commands in your terminal:

   ```bash
   # Start PostgreSQL
   docker run --name lightx-pg -e POSTGRES_PASSWORD=password -e POSTGRES_DB=lightx_test -p 5432:5432 -d postgres

   # Start MySQL
   docker run --name lightx-mysql -e MYSQL_ROOT_PASSWORD=password -e MYSQL_DATABASE=lightx_test -p 3306:3306 -d mysql
   ```

   _(💡 If you do not have Docker installed, you can grab it from docker.com. Alternatively, if you already host your own local databases, simply update the `mysql://...` and `postgres://...` connection strings inside the `.env` file natively)_.

### Step 3: Start the LightX Server

Now that Rust is correctly installed and the databases are ready, you can safely boot the API.

1. **Navigate to the `lightx-api` project directory:**

   ```bash
   cd lightx-api
   ```

2. **Launch the development compiler:**

   ```bash
   cargo run
   ```

   > 💡 **What is happening here?** During this automated boot sequence, the framework scans your TOML models, deduces all necessary SQL queries (DAOs), orchestrates your routers (AOP), compiles everything into a binary, and finally boots a robust, secure asynchronous server on port `3000`.

   > ✅ You will know everything executed perfectly when you spot this message in your terminal: `Démarrage de LightX API (JSON REST)!`. (The server will block this terminal running as a daemon, this is completely normal!).

### Step 4: Testing the Endpoints

Congratulations, the server is live! Now, open a **strictly new terminal window** (or simply use your web browser) to poke the three generated routes and interact dynamically with the databases.

- **To run the PostgreSQL demonstration:**
  ```bash
  curl http://localhost:3000/postgres/DbDemo
  ```
- **To run the MySQL / MariaDB demonstration:**
  ```bash
  curl http://localhost:3000/mysql/DbDemo
  ```
- **To run the SQLite demonstration:**
  ```bash
  curl http://localhost:3000/sqlite/DbDemo
  ```

🎉 **Expected result:** Each request instantly yields a deeply detailed JSON dashboard ("status: success"). This JSON proves the successful and ultra-fast background execution of complex queries (Batch Inserts, Native Pagination, Transactional Integrity).

---

## 🧠 Understanding the Framework: Excellence in Architecture

This repository operates way above standard monolithic routers. It embeds carefully defined guidelines towards **Pedagogical and Pure Code State of the Art (SOTA) methodologies** adopted natively by LightX engine components.
Master the underlying magic below to scale architectures fearlessly.

### 1. The Power of Macros (True Zero-Cost Generation)

LightX strictly abandons _runtime reflections_, completely bypassing the delays found heavily in classical monolithic enterprise layers.
During the precise `cargo run` compilation timeframe, the core parses TOML logic tables and SQL footprints into immense scopes of **highly-optimized, low-level Rust raw-native code** via Daox.

- **Production Consequence**: Instantaneous routing allocations (`O(1)` match limits), total immunity against accidental memory-leak crashes and solid intrinsic guards fighting modern SQL injections.

### 2. Aspect-Oriented Programming (AOP) Pipelining

Inside this template logic, endpoints logic behaves solely via declarations defined securely from the `/handlers` map layouts.
The generated AOP pipeline works globally as a perfect, faultless sentry taking responsibilities of:

- Unwrapping and statically parsing raw body-JSON data.
- Handling mapping routes efficiently via lock-free operations.
- Queued executions natively resolving routines through your customized **Business Object (BO)** sequences.
- Ultimate SQL Safeguards: Handling all `.COMMIT()` calls globally transparently, firing forced `.ROLLBACK()` triggers whenever single sequences fails safely.

### 3. Component Architecture Matrix (Severe Segregation)

Building tools comes easy providing humans distinct clean boundaries:

1. **The Core Data-Access (DAO)**: Manufactured under-the-hood libraries (`Daox`). Performs complex interactions and serves zero-boilerplate native structs APIs providing (Inserts, Paging, Cursors tracking, Stream evaluations).
2. **The Logic Business Object Layer (BO)** : Nested around `src/bo/`. **This is precisely the domain where you code!** Forget data-structures and focus purely algorithmically, triggering operations effortlessly leveraging the auto-assigned DAOs.
3. **The Data Bus (RequestContext)** : Eradicating dangerous global static environments. Context Factorials extracts `.env` pointers mapped safely internally (`ctx.postgres_pool`) packed into purely asynchronous singletons strictly handed out to the current request execution, collected seamlessly thread-safe utilizing explicit RAII guarantees standard scopes upon process completions.

### 4. Zero Panics, Realizing 100% Rust Purity

While running tests inside the explicit layout (witnessed across functions nested in `DbDemoBo.rs` sources), you actively handle mapping logic safely enclosed over customized enum structures named `AppError`.
Crucial framework methodologies force-evicts completely dangerous runtime triggers known universally as standard `panic!()` or internal manual `unwrap()`.

**The State of the Art execution process (SOTA)** : Be it a broken routing path, null pointer traces dropping, or severe offline Database outages detected — structural paradigms intercepts natively everything down the chain seamlessly! The `?` catch mapper encapsulates broken behaviors to bounce back completely readable standardized error JSON traces rendering beautifully up to the Frontend SPA components natively making promises the Core Thread Pool guarantees un-crashed, scalable API deliveries round-the-clock seamlessly.

---

**Congratulations**, you successfully transitioned decoding native internal engines and mysteries fueling LightX uncompromised, top-tier reliability pipelines!
