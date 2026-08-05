//! # LightX API Showcase Application
//!
//! This is the main entry point for the `lightx-api` application, demonstrating the ability of
//! the LightX framework to serve ultra-fast RESTful JSON data with near zero-cost abstractions.
//!
//! ## Educational Value
//! - Demonstrates zero-dependency JSON APIs.
//! - Uses auto-generated, strictly-typed Database Access Objects (DAOs) matching DB schemas.
//! - Bootstraps the application via a static radix-tree router and AOP Context interceptors.

use lightx::server;
use std::sync::Arc;

/// The Business Object (BO) layer representing the actionable REST endpoints API logic.
pub mod bo;

/// The Database Access Object (DAO) layer generated at build-time by `daox`.
/// Provides completely safe, statically-typed query methods tailored for PostgreSQL, MySQL, and SQLite.
pub mod daox_generated;
pub use daox_generated::*;

// Include the dynamically generated API Handlers and Core router implementations at build-time.
// These endpoints are configured via descriptive `.toml` files in the `handlers/` directory.
include!(concat!(env!("OUT_DIR"), "/lightx_handlers_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/lightx_core_generated.rs"));

/// The asynchronous main function, acting as the entry point of the API server.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables securely from the `.env` configuration file.
    dotenvy::dotenv().ok();

    // Initialize the high-performance asynchronous O(1) logging infrastructure.
    // Highly resilient under massive concurrency spikes since disk-writes are non-blocking.
    let log_dir = std::env::var("LIGHTX_LOG_DIR").unwrap_or_else(|_| "./log".to_string());
    let _ = lightx::logger::init(&log_dir);

    lightx::logger::info("Starting the LightX API (JSON REST)!".to_string());

    // Context Factory: Responsible for dynamically spawning and injecting DB Pools
    // into incoming HTTP requests seamlessly.
    let factory = Arc::new(crate::AppContextFactory::new().await?);

    // Application Router: Resolves paths into handler invocations using a static radix-tree configuration.
    let router = Arc::new(crate::AppRouter {
        factory: factory.clone(),
    });

    // Define the IPv4 Socket Address for the REST API server to listen on port 3000.
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));

    // Bootstrap, bind connections, and listen actively.
    server::listen(addr, factory, router).await?;

    Ok(())
}
