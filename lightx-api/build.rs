use daox::DaoGenerator;
use lightx::core_generator::CoreGenerator;
use lightx::handler_generator::HandlerGenerator;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    println!("cargo:rerun-if-changed=schema");
    println!("cargo:rerun-if-changed=handlers");
    println!("cargo:rerun-if-changed=templates");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let schema_dir = PathBuf::from(manifest_dir.clone()).join("schema");
    let out_dir = env::var("OUT_DIR").unwrap();

    let daox_out_dir = PathBuf::from(manifest_dir.clone())
        .join("src")
        .join("daox_generated");
    std::fs::create_dir_all(&daox_out_dir).unwrap_or_default();
    let dao_gen = DaoGenerator::new(schema_dir.to_str().unwrap(), daox_out_dir.to_str().unwrap());

    // Offline DB approach (use the TOML files)
    if let Some(url) = env::var("DATABASE_URL")
        .ok()
        .filter(|u| u != "offline" && env::var("CI").is_err())
    {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let _ = dao_gen.introspect(&url).await;
        });
    }

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
