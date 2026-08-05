# LightX API - Demonstration and architecture guide

Welcome to the demonstration API of the LightX framework. This project serves as a technological and pedagogical showcase to understand how to interact with the incredible capabilities of the automated code generation engine powered by **LightX** and **Daox**.

[English](README.md) | [Français](README.fr.md)

---

## Quick start (step-by-step tutorial)

This guide is designed to be accessible to everyone, even absolute beginners. Please follow every step carefully.

### Step 1: Prepare your development environment

To run this project, you need the foundational Rust programming tools.

- **Install Rust**: If you haven't yet, install the compiler by opening your terminal and running the official command:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  > *( Crucial for beginners: Once the installation finishes, carefully restart your terminal or execute `source $HOME/.cargo/env` to activate the `cargo` command)._

### Step 2: Prepare the databases

This demonstration project showcases LightX's power on 3 databases simultaneously. Therefore, the API requires these databases to be reachable before starting.

1. **SQLite**: Runs natively in standard RAM (`sqlite::memory:`). **No installation required!**
2. **PostgreSQL & MySQL**: The server will seamlessly attempt to connect using the credentials present in the `.env` file (`localhost:5432` and `localhost:3306`).

    **The simplest solution (via Docker)**: If you possess Docker, you can instantly spin up and host both databases with the proper credentials using these two simple commands in your terminal:

   ```bash
   # Start PostgreSQL
   docker run --name lightx-pg -e POSTGRES_PASSWORD=password -e POSTGRES_DB=lightx_test -p 5432:5432 -d postgres

   # Start MySQL
   docker run --name lightx-mysql -e MYSQL_ROOT_PASSWORD=password -e MYSQL_DATABASE=lightx_test -p 3306:3306 -d mysql
   ```

   > *( If you do not have Docker installed, you can grab it from docker.com. Alternatively, if you already host your own local databases, simply update the `mysql://...` and `postgres://...` connection strings inside the `.env` file natively)*.

### Step 3: Start the LightX server

Now that Rust is correctly installed and the databases are ready, you can safely boot the API.

1. **Navigate to the `lightx-api` project directory:**

   ```bash
   cd lightx-api
   ```

2. **Launch the development compiler:**

   ```bash
   cargo run
   ```

   >  **What is happening here?** During this automated boot sequence, the framework scans your TOML models located in the `schema/` directory, deduces all necessary SQL queries (DAOs), orchestrates your routers (AOP), compiles everything into a binary, and finally boots a robust, secure asynchronous server on port `3000`.

   >  You will know everything executed perfectly when you spot this message in your terminal: `Démarrage de LightX API (JSON REST)!`. (The server will block this terminal running as a daemon, this is completely normal!).

### Step 4: Testing the endpoints

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

 **Expected result:** Each request instantly yields a deeply detailed JSON dashboard ("status: success"). This JSON proves the successful and ultra-fast background execution of complex queries (Batch Inserts, Native Pagination, Transactional Integrity).

> *( **Pedagogical Import Note**: Wondering how your empty Docker containers suddenly obtained their tables? During your curl request, the "DbDemoBo" Business Object flawlessly imported and executed the raw SQL files located in the `migrations/` folder on-the-fly!)*

---

## Tutorial: Creating your own LightX API from scratch

Creating an end-to-end API with **LightX** is designed to be extremely fast, secure, and focused on modern engineering. The framework's philosophy is: _Declarative first, Pure Business Code, and Zero Runtime Reflection_.

Here is the step-by-step pedagogical guide.

### Phase 1: Architecture and project startup

**1. Rust Project Initialization**
Start by creating the standard project:

```bash
cargo new my_lightx_api
cd my_lightx_api
```

**2. Dependencies (`Cargo.toml`)**
Open `Cargo.toml` and rigorously define the required libraries. `lightx` powers the async server, and `daox` enables code generation.

```toml
[dependencies]
lightx = "0.1"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.9", features = ["runtime-tokio", "postgres"] }
dotenvy = "0.15"

[build-dependencies]
daox = { version = "0.2", features = ["postgres"] }
lightx = "0.1"
tokio = { version = "1", features = ["full"] }
dotenvy = "0.15"
```

**3. Environment Variables (`.env`)**
At the root of your project, create a `.env` file. This acts as the single source of truth for network accesses.

```env
POSTGRES_DATABASE_URL=postgres://user:password@localhost:5432/my_database
```

### Phase 2: Data modeling (the DAO layer)

The absolute strength of LightX lies in its **Daox** generator, which embraces an unwavering _Database-first_ approach.

**1. Model in SQL**
Create your tables physically in your Postgres database via your daily SQL utility:

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL
);
```

**2. Target Configuration (`schema/databases.toml`)**
Create the `schema/` directory followed by the `schema/databases.toml` file to instruct the compiler targets:

```toml
[postgres]
dialect = "postgres"
```

**3. The Generation Script (`build.rs`)**
At the root of the project (next to `Cargo.toml`), create the `build.rs` script. This engine introspects tables and structures routers before the official compilation triggers:

```rust
use daox::DaoGenerator;
use lightx::core_generator::CoreGenerator;
use lightx::handler_generator::HandlerGenerator;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let out_dir = env::var("OUT_DIR").unwrap();

    // 1. DAO Introspection and generation
    let dao_gen = DaoGenerator::new("schema", &out_dir);
    if let Ok(url) = env::var("POSTGRES_DATABASE_URL") {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let _ = dao_gen.introspect(&url).await;
        });
    }
    dao_gen.generate_dao()?;

    // 2. Router Generation (Handlers & Core)
    HandlerGenerator::new("handlers", &out_dir).generate_handlers()?;
    CoreGenerator::new("handlers", "i18n", &out_dir).generate_core()?;

    Ok(())
}
```

### Phase 3: Router creation via AOP (handlers)

Unlike classic frameworks, LightX utilizes Aspect-Oriented Programming declaratively, completely eliminating "spaghetti code".

**Declare the Route (`handlers/CreateUser.toml`)**
Create the `handlers/` directory, then the `handlers/CreateUser.toml` file:

```toml
name = "CreateUser"
method = "POST"
path = "/users/create"
bo_class = "crate::bo::create_user::CreateUserBo"
```

During compilation, the framework reads this file to forge a secure controller parsing the HTTP request and invoking your layout naturally.

### Phase 4: Writing the business logic (BO)

Forget network mechanics, strictly focus on algorithmics.

**1. Declare the Module**
Create a `src/bo/` directory and build the `src/bo/mod.rs` file exposing your module:

```rust
pub mod create_user;
```

**2. Write the Logic (`src/bo/create_user.rs`)**
Then, author the `src/bo/create_user.rs` file inherently mapped by your Handler:

```rust
use lightx::core::AppError;
use lightx::ext::hyper::Response;

pub struct CreateUserBo;

impl CreateUserBo {
    pub async fn execute(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<String>, AppError> {

        // 1. Lazy database connection fetching through the centralized context
        let pool = &ctx.postgres_pool;

        // 2. Employ the native Rust model (Automatically generated via Daox)
        let new_user = crate::PostgresUsers {
            id: 0,
            email: "my_email@domain.com".to_string(),
        };

        // 3. Insertion guarded with strict framework error mechanisms
        let user_id = new_user.insert(pool).await.map_err(|e| AppError::DatabaseError {
            msg: e.to_string(), file: file!(), line: line!()
        })?;

        // 4. Formalize the HTTP response output
        Ok(Response::new(format!("User successfully generated bearing ID {}", user_id)))
    }
}
```

### Phase 5: Program Entry (`src/main.rs`)

Open your `src/main.rs` file and entirely copy the boot code below. This connects internally auto-generated components to standard operations.

```rust
use std::sync::Arc;
use lightx::server;

// Direct injection of native code forged during the build layer (DAO, Handlers, Core)
include!(concat!(env!("OUT_DIR"), "/daox_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/lightx_handlers_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/lightx_core_generated.rs"));

// The tree hierarchy embracing business logic
pub mod bo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok(); // Inject network constraints
    lightx::logger::init("./log"); // Jumpstart asynchronous logging tasks

    lightx::logger::info("LightX Powered API Activating!".to_string());

    // Context Factory: Instantiates structured database pools independently
    let factory = Arc::new(crate::AppContextFactory::new().await?);

    // Precompiled router evaluating natively without overhead limits O(1)
    let router = Arc::new(crate::AppRouter { factory: factory.clone() });

    // Engaging listener on active port 3000
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    server::listen(addr, factory, router).await?;

    Ok(())
}
```

Execute `cargo run`, and your extreme-performance API stands proudly online!

---

## Understanding the framework: excellence in architecture

This repository operates way above standard monolithic routers. It embeds carefully defined guidelines towards **Pedagogical and Pure Code methodologies** adopted natively by LightX engine components.
Master the underlying magic below to scale architectures fearlessly.

![Global Architecture](assets/architecture_en.svg)

### 1. The power of macros (true zero-cost generation)

LightX strictly abandons _runtime reflections_, completely bypassing the delays found heavily in classical monolithic enterprise layers.
During the precise `cargo run` compilation timeframe, the core parses TOML logic tables and SQL footprints into immense scopes of **highly-optimized, low-level Rust raw-native code** via Daox.

- **Production Consequence**: Instantaneous routing allocations (`O(1)` match limits), total immunity against accidental memory-leak crashes and solid intrinsic guards fighting modern SQL injections.

### 2. Aspect-oriented programming (AOP) pipelining

Inside this template logic, endpoints logic behaves solely via declarations defined securely from the `/handlers` map layouts.
The generated AOP pipeline works globally as a perfect, faultless sentry taking responsibilities of:

- Unwrapping and statically parsing raw body-JSON data.
- Handling mapping routes efficiently via lock-free operations.
- Queued executions natively resolving routines through your customized **Business Object (BO)** sequences.
- Ultimate SQL Safeguards: Handling all `.COMMIT()` calls globally transparently, firing forced `.ROLLBACK()` triggers whenever single sequences fails safely.

![AOP Lifecycle](assets/aop_en.svg)

### 3. Component architecture matrix (severe segregation)

Building tools comes easy providing humans distinct clean boundaries:

1. **The Core Data-Access (DAO)**: Manufactured under-the-hood libraries (`Daox`). Performs complex interactions and serves zero-boilerplate native structs APIs providing (Inserts, Paging, Cursors tracking, Stream evaluations).
2. **The Logic Business Object Layer (BO)** : Nested around `src/bo/`. **This is precisely the domain where you code!** Forget data-structures and focus purely algorithmically, triggering operations effortlessly leveraging the auto-assigned DAOs.
3. **The RequestContext (The Bus)**: LightX vehemently forbids risky global states. The Context Factory gathers your `.env` drivers (e.g., `ctx.postgres_pool`) into an extremely lightweight lazy environment, delivered directly into your Business Objects and magically wiped out from memory via strict Rust temporal rules (RAII) right after HTTP cycle bindings.

![Context Lifecycle](assets/context_en.svg)

### 4. Zero panics, realizing 100% Rust purity

While running tests inside the explicit layout (witnessed across functions nested in `DbDemoBo.rs` sources), you actively handle mapping logic safely enclosed over customized enum structures named `AppError`.
Crucial framework methodologies force-evicts completely dangerous runtime triggers known universally as standard `panic!()` or internal manual `unwrap()`.

**The native resilience execution process** : Be it a broken routing path, null pointer traces dropping, or severe offline Database outages detected — structural paradigms intercepts natively everything down the chain seamlessly! The `?` catch mapper encapsulates broken behaviors to bounce back completely readable standardized error JSON traces rendering beautifully up to the Frontend SPA components natively making promises the Core Thread Pool guarantees un-crashed, scalable API deliveries round-the-clock seamlessly.

---

**Congratulations**, you successfully transitioned decoding native internal engines and mysteries fueling LightX uncompromised, top-tier reliability pipelines!
