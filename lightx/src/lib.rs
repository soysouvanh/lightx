//!  `lightx`: The "Zero-Overhead" Pedagogical Framework.
//!
//! LightX is an educational yet production-ready framework built on the
//! "Database-First" paradigm. It relies on strict code generation at compile time
//! (`build.rs`) to avoid runtime reflection, guaranteeing absolute type safety
//! and maximum performance.
//!
//! # Architecture
//! - `core`: Contains the global error management (`AppError`) and bail macros.
//! - `dao_generator`: The build-time engine that introspects your MySQL database,
//!   generates TOML dictionaries, and writes Rust `structs` directly into `OUT_DIR`.
//! - `server`: The hardened Hyper HTTP server with anti-OOM, anti-Slowloris, TLS 1.3,
//!   and military-grade security headers.

#![deny(clippy::undocumented_unsafe_blocks)]

// 🛡️ Military Strict Versioning Control
// Enforces the absolute physical presence of `CHANGELOG.md` at compile time.
// Any `cargo publish` will mathematically crash if the documentation is decoupled.
const _CHANGELOG_VALIDATION: &str = include_str!("../CHANGELOG.md");

pub mod core;
pub mod core_generator;
pub mod handler_generator;
pub mod logger;
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
