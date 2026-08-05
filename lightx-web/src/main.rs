//! # LightX Web Showcase Application
//!
//! This is the main entry point for the `lightx-web` showcase. It demonstrates how to initialize
//! and run a highly performant, State-of-the-Art web application using the LightX framework.
//!
//! ## Educational Value
//! This file shows the simplicity of the LightX footprint:
//! 1. Inclusion of automatically generated modules (Templates, DAOs, and Routing).
//! 2. Initialization of the asynchronous O(1) logger.
//! 3. Instantiation of the overarching Context Factory and Router.
//! 4. Bootstrapping the hyper HTTP server.

use lightx::server;
use std::sync::Arc;

/// The template module generated at build-time by `tmplx`.
/// Includes statically optimized buffer allocations and rendering macros.
#[macro_use]
#[allow(unused_variables)]
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/template_gen.rs"));
}

/// The Business Object (BO) layer representing actionable logic.
pub mod bo;

/// The Database Access Object (DAO) layer generated at build-time by `daox`.
/// Provides completely safe, statically-typed query methods.
pub mod daox_generated;
pub use daox_generated::*;

// Include the generated Handlers and Core router implementations.
// These are dynamically generated based on `.toml` configurations
// in the `handlers/` directory, adhering to Convention over Configuration.
include!(concat!(env!("OUT_DIR"), "/lightx_handlers_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/lightx_core_generated.rs"));

/// The asynchronous main function, entry point of the Toki-based application.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables securely from the `.env` file.
    dotenvy::dotenv().ok();

    // Initialize the high-performance asynchronous O(1) logging infrastructure.
    // The logger writes efficiently to disk while preventing memory bottlenecks under heavy load.
    let log_dir = std::env::var("LIGHTX_LOG_DIR").unwrap_or_else(|_| "./log".to_string());
    let _ = lightx::logger::init(&log_dir);

    lightx::logger::info("Starting the End-to-End Showcase Server!".to_string());

    // Context Factory: Responsible for generating request-specific contexts.
    // It dynamically configures DB Connection Pools according to `.env` specifications.
    let factory = Arc::new(crate::AppContextFactory::new().await?);

    // Application Router: Resolves paths into handler invocations using an efficient radix-tree setup.
    let router = Arc::new(crate::AppRouter {
        factory: factory.clone(),
    });

    // Define the IPv4 Socket Address for incoming HTTP connections.
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8081));

    // Bootstrap and listen actively.
    server::listen(addr, factory, router).await?;

    Ok(())
}
