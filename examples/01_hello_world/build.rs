use daox::DaoGenerator;
use lightx::core_generator::CoreGenerator;
use lightx::handler_generator::HandlerGenerator;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let schema_dir = PathBuf::from(manifest_dir.clone()).join("schema");
    let out_dir = env::var("OUT_DIR").unwrap();

    let dao_gen = DaoGenerator::new(schema_dir.to_str().unwrap(), &out_dir);

    // =====================================================================
    // 1. DATABASE-FIRST: Introspection and TOML Dictionary Generation
    // =====================================================================
    if let Ok(url) = env::var("DATABASE_URL") {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            dao_gen
                .introspect(&url)
                .await
                .expect("Failed to introspect MySQL database");
        });
    } else {
        println!("cargo:warning= DATABASE_URL not set in environment. Skipping database introspection. Offline DB Mode active.");
    }

    // =====================================================================
    // 2. METAPROGRAMMING PIPELINE: Rust DAO Generation
    // =====================================================================
    dao_gen.generate_dao()?;

    let handlers_dir = PathBuf::from(manifest_dir.clone()).join("handlers");
    let handler_gen = HandlerGenerator::new(handlers_dir.to_str().unwrap(), &out_dir);
    handler_gen.generate_handlers()?;

    let i18n_dir = PathBuf::from(manifest_dir.clone()).join("i18n");
    let core_gen = CoreGenerator::new(
        handlers_dir.to_str().unwrap(),
        i18n_dir.to_str().unwrap(),
        &out_dir,
    );
    core_gen.generate_core()?;

    Ok(())
}
