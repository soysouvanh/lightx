#![allow(clippy::all)]

pub mod bo;

include!(concat!(env!("OUT_DIR"), "/lightx_dao_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/lightx_handlers_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/lightx_core_generated.rs"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    println!("Starting LightX App...");

    // Connect to database using agnostic factory
    let factory = std::sync::Arc::new(crate::AppContextFactory::new().await?);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    let router = std::sync::Arc::new(crate::AppRouter);

    lightx::server::listen(addr, factory, router).await?;

    Ok(())
}
