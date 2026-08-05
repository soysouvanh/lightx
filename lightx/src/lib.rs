//! # `lightx`: The Absolute Zero-Overhead Framework.
//!
//! LightX is an educational yet intensely production-ready HTTP and Web framework.
//! Built around the absolute paradigm of "Database-First" and Aspect-Oriented Programming (AOP),
//! it entirely avoids runtime reflection and dynamic dispatch overheads by generating highly
//! optimized static routers, database models (via `daox`), and handlers at compile time (`build.rs`).
//!
//! ## Core Philosophy
//! - **Mathematical Certainty:** By utilizing `build.rs` to generate strict structures (stored in `OUT_DIR`), the framework ensures that everything from databases to API schemas is guaranteed by the Rust compiler.
//! - **Zero Dependencies at Runtime:** Unlike heavy ecosystems, LightX strips away orchestrators. The generated handlers bind directly to native hyper streams utilizing `moka` caches and `tokio` O(1) structures.
//!
//! ## Modules Overview
//! - [`core`]: Provides the globally propagated `AppError`, safe bail macros, and background task orchestrators.
//! - [`core_generator`]: Build-time engine generating the static Radix Tree (O(1) router) and `RequestContext`.
//! - [`handler_generator`]: Build-time engine translating handler TOML definitions into exact Typed Extractor workflows.
//! - [`logger`]: High-performance asynchronous MPSC-based logging (mathematically decoupling disk I/O from thread loops).
//! - [`server`]: Hardened Hyper HTTP server enforcing DoS mitigations, strict TLS versions, and memory safety invariants.

#![deny(clippy::undocumented_unsafe_blocks)]

//  Military Strict Versioning Control
// Enforces the absolute physical presence of `CHANGELOG.md` at compile time.
// Any `cargo publish` will mathematically crash if the documentation is decoupled.
const _CHANGELOG_VALIDATION: &str = include_str!("../CHANGELOG.md");

/// Core Application mechanisms (Errors, Enums, Macros, Tasks).
pub mod core;

/// Build-time engine yielding O(1) Static Routers and Global Context allocation logic.
pub mod core_generator;

/// Build-time engine parsing TOML ASTs into completely statically typed HTTP Handlers.
pub mod handler_generator;

/// O(1) Lock-free Asynchronous disk logger orchestrator protecting standard output.
pub mod logger;

/// Bare-metal `hyper` HTTP/HTTPS listener embedding military-grade connection constraints.
pub mod server;

/// Extensions re-exported for the generated code.
/// Only production-necessary crates are exposed.
pub mod ext {
    pub use bytes;
    pub use futures_util;
    pub use http_body_util;
    pub use hyper;
    pub use hyper_tungstenite;
    pub use hyper_util;
    pub use jsonwebtoken;
    pub use matchit;
    pub use moka;
    pub use opentelemetry;
    pub use opentelemetry_http;
    pub use regex;
    pub use rustls;
    pub use rustls_pemfile;
    pub use serde_json;
    pub use tokio;
    pub use tokio_rustls;
    pub use tokio_tungstenite;
    pub use tokio_util;
    pub use tower;
    pub use tracing;
    pub use tracing_opentelemetry;
}
