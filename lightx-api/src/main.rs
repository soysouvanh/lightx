use lightx::server;
use std::sync::Arc;

pub mod bo;

pub mod daox_generated;
pub use daox_generated::*;
include!(concat!(env!("OUT_DIR"), "/lightx_handlers_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/lightx_core_generated.rs"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialise l'infrastructure de journalisation asynchrone (O(1))
    let log_dir = std::env::var("LIGHTX_LOG_DIR").unwrap_or_else(|_| "./log".to_string());
    let _ = lightx::logger::init(&log_dir);

    lightx::logger::info("Démarrage de LightX API (JSON REST)!".to_string());

    let factory = Arc::new(crate::AppContextFactory::new().await?);
    let router = Arc::new(crate::AppRouter {
        factory: factory.clone(),
    });

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));

    server::listen(addr, factory, router).await?;

    Ok(())
}
