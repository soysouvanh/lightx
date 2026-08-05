use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "lightx",
    version,
    author,
    about = "LightX Framework Automation CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new LightX project from scratch
    New {
        /// Name of the project to create
        project_name: String,
    },
    /// Automatisation tooling for existing projects
    Add {
        #[command(subcommand)]
        target: AddCommands,
    },
    /// Execute database SQL migrations securely from the `migrations/` directory
    Migrate,
}

#[derive(Subcommand)]
enum AddCommands {
    /// Add a new handler to the project
    Handler {
        /// HTTP Method (GET, POST, PUT, DELETE, etc.)
        method: String,
        /// URI Path (e.g. /api/users)
        uri: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { project_name } => scaffold_new_project(project_name),
        Commands::Add { target } => match target {
            AddCommands::Handler { method, uri } => scaffold_handler(method, uri),
        },
        Commands::Migrate => run_migrations(),
    }
}

fn scaffold_new_project(project_name: &str) {
    let project_dir = Path::new(project_name);
    if project_dir.exists() {
        eprintln!("Error: Directory '{}' already exists.", project_name);
        std::process::exit(1);
    }

    // 1. Create directory structure
    fs::create_dir_all(project_dir.join("schema")).expect("Failed to create schema directory");
    fs::create_dir_all(project_dir.join("migrations"))
        .expect("Failed to create migrations directory");
    fs::create_dir_all(project_dir.join("handlers")).expect("Failed to create handlers directory");
    fs::create_dir_all(project_dir.join("i18n")).expect("Failed to create i18n directory");
    fs::create_dir_all(project_dir.join("src/bo")).expect("Failed to create src/bo directory");

    // 2. Write minimal Cargo.toml
    let cargo_toml = format!(
        r#"[workspace]

[package]
name = "{}"
version = "0.2.0"
edition = "2024"

[dependencies]
lightx = {{ path = "../lightx" }}
tokio = {{ version = "1.37.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
sqlx = {{ version = "0.8", features = ["runtime-tokio", "mysql"] }}
dotenvy = "0.15"

[build-dependencies]
lightx = {{ path = "../lightx" }}
dotenvy = "0.15"

[dev-dependencies]
lightx-macro = {{ path = "../lightx-macro" }}
"#,
        project_name
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    // 3. Write minimal Databases schema stub
    let databases_toml = r#"[default]
connection_string_env = "DATABASE_URL"
"#;
    fs::write(project_dir.join("schema/databases.toml"), databases_toml)
        .expect("Failed to write schema/databases.toml");

    // 4. Write minimal main.rs
    let main_rs = r#"use sqlx::MySqlPool;
use std::env;
use std::sync::Arc;

pub mod bo;

pub mod daox_generated;
pub use daox_generated::*;
include!(concat!(env!("OUT_DIR"), "/lightx_handlers_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/lightx_core_generated.rs"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    println!("Starting LightX App...");
    
    // Connect to database
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "mysql://root:root@127.0.0.1:3306/my_database".to_string());
    let pool = MySqlPool::connect(&db_url).await?;
    
    let factory = Arc::new(AppContextFactory::new().await?);
    
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    let router = Arc::new(AppRouter { factory: factory.clone() });
    
    lightx::server::listen(addr, factory, router).await?;
    
    Ok(())
}
"#;
    fs::write(project_dir.join("src/main.rs"), main_rs).expect("Failed to write main.rs");

    // 5. Write minimal build.rs
    let build_rs = r#"use lightx::core_generator::CoreGenerator;
use daox::DaoGenerator;
use lightx::handler_generator::HandlerGenerator;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let schema_dir = PathBuf::from(manifest_dir.clone()).join("schema");
    let out_dir = env::var("OUT_DIR").unwrap();

    let daox_out_dir = PathBuf::from(manifest_dir.clone()).join("src").join("daox_generated");
    std::fs::create_dir_all(&daox_out_dir).unwrap_or_default();

    let dao_gen = DaoGenerator::new(
        schema_dir.to_str().unwrap(),
        daox_out_dir.to_str().unwrap()
    );

    // =====================================================================
    // 1. DATABASE-FIRST: Introspection and TOML Dictionary Generation
    // =====================================================================
    if let Ok(url) = env::var("DATABASE_URL") {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            dao_gen
                .introspect_mysql(&url)
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
    let handler_gen = HandlerGenerator::new(
        handlers_dir.to_str().unwrap(),
        &out_dir
    );
    handler_gen.generate_handlers()?;

    let i18n_dir = PathBuf::from(manifest_dir.clone()).join("i18n");
    let core_gen = CoreGenerator::new(
        handlers_dir.to_str().unwrap(),
        i18n_dir.to_str().unwrap(),
        &out_dir
    );
    core_gen.generate_core()?;

    Ok(())
}
"#;
    fs::write(project_dir.join("build.rs"), build_rs).expect("Failed to write build.rs");

    println!(
        " Successfully initialized LightX project '{}'",
        project_name
    );
}

fn scaffold_handler(method: &str, uri: &str) {
    let method = method.to_uppercase();
    if !["GET", "POST", "PUT", "DELETE", "PATCH"].contains(&method.as_str()) {
        eprintln!("Error: Unsupported HTTP method '{}'", method);
        std::process::exit(1);
    }

    // Example: /api/users -> ApiUsers
    let uri_clean = uri.trim_start_matches('/');
    let uri_clean = if uri_clean.is_empty() {
        "index"
    } else {
        uri_clean
    };

    let segments_raw: Vec<&str> = uri_clean.split('/').filter(|s| !s.is_empty()).collect();
    if segments_raw.is_empty() {
        eprintln!("Error: Valid URI segments missing.");
        std::process::exit(1);
    }

    let mut toml_dir = Path::new("handlers").to_path_buf();
    let mut bo_dir = Path::new("src/bo").to_path_buf();

    // Process all segments except the last one to build directories
    for seg in segments_raw.iter().take(segments_raw.len() - 1) {
        let seg = seg.replace("-", "_").replace("{", "").replace("}", "");
        toml_dir.push(&seg);
        bo_dir.push(&seg);
    }

    // Process the last segment for the file name
    let last_seg = segments_raw.last().unwrap();
    let mut file_base = String::new();
    let mut capitalize_next = true;
    for c in last_seg.chars() {
        if c == '-' || c == '_' || c == '{' || c == '}' || c == '.' {
            capitalize_next = true;
        } else if capitalize_next {
            file_base.push_str(&c.to_uppercase().to_string());
            capitalize_next = false;
        } else {
            file_base.push(c);
        }
    }

    let capitalized_method = {
        let mut chars = method.chars();
        match chars.next() {
            Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase()),
            None => "".to_string(),
        }
    };
    file_base.push_str(&capitalized_method);

    // Also determine the full handler name for struct names, by capitalizing ALL segments
    let mut handler_name = String::new();
    for seg in &segments_raw {
        let mut capitalize_next = true;
        for c in seg.chars() {
            if c == '-' || c == '_' || c == '{' || c == '}' || c == '.' {
                capitalize_next = true;
            } else if capitalize_next {
                handler_name.push_str(&c.to_uppercase().to_string());
                capitalize_next = false;
            } else {
                handler_name.push(c);
            }
        }
    }
    handler_name.push_str(&capitalized_method);

    let toml_path = toml_dir.join(format!("{}.toml", file_base));
    let bo_filename = format!("{}_bo", file_base.to_lowercase());
    let bo_path = bo_dir.join(format!("{}.rs", bo_filename));

    if toml_path.exists() {
        eprintln!(
            "Error: Handler specification '{}' already exists.",
            toml_path.display()
        );
        std::process::exit(1);
    }

    if bo_path.exists() {
        eprintln!(
            "Error: Business Object file '{}' already exists.",
            bo_path.display()
        );
        std::process::exit(1);
    }

    // Calculate module path for standard AOP processing mapping
    let mut mod_path_items = vec![];
    for seg in segments_raw.iter().take(segments_raw.len() - 1) {
        mod_path_items.push(seg.replace("-", "_").replace("{", "").replace("}", ""));
    }
    mod_path_items.push(bo_filename.clone());
    let bo_import_path = mod_path_items.join("::");

    // 1. Generate TOML
    let toml_content = format!(
        r#"[metadata]
version = "1.0.0"
description = "Auto-generated {} handler"

[route]
method = "{}"
uri = "{}"
authentication = "Public"
rate_limit = 100

[parameters]
# Define your parameters here, mapped to data dictionary (e.g. email = "users.email")
# Empty string means arbitrary parameter without DB column binding
# email = ""

[pipeline]
validations = []
processings = ["{}::{}::execute"]
"#,
        handler_name, method, uri, bo_import_path, handler_name
    );

    fs::create_dir_all(&toml_dir).expect("Failed to create handlers directory");
    fs::write(&toml_path, toml_content).expect("Failed to write handler TOML");

    // 2. Generate BO File
    let bo_content = format!(
        r#"use crate::RequestContext;

pub struct {} {{}}

impl {} {{
    pub async fn execute(_ctx: &mut RequestContext) -> Result<(), lightx::core::AppError> {{
        println!("Executing {} ...");
        // TODO: Implement your business logic here
        
        Ok(())
    }}
}}
"#,
        handler_name, handler_name, handler_name
    );

    fs::create_dir_all(&bo_dir).expect("Failed to create src/bo directory");
    fs::write(&bo_path, bo_content).expect("Failed to write BO file");

    // 3. Inject mod into src/bo/mod.rs (recursively for nested directories)
    let mut current_dir = Path::new("src/bo").to_path_buf();
    for seg in segments_raw.iter().take(segments_raw.len() - 1) {
        let seg = seg.replace("-", "_").replace("{", "").replace("}", "");
        let mod_rs_path = current_dir.join("mod.rs");
        let mod_stmt = format!("pub mod {};\n", seg);

        if mod_rs_path.exists() {
            let content = fs::read_to_string(&mod_rs_path).unwrap();
            if !content.contains(&mod_stmt) {
                let new_content = format!("{}{}", content, mod_stmt);
                fs::write(&mod_rs_path, new_content).expect("Failed to update mod.rs");
            }
        } else {
            fs::write(&mod_rs_path, mod_stmt).expect("Failed to create mod.rs");
        }
        current_dir.push(&seg);
    }

    let final_mod_rs_path = current_dir.join("mod.rs");
    let mod_stmt = format!("pub mod {};\n", bo_filename);
    if final_mod_rs_path.exists() {
        let content = fs::read_to_string(&final_mod_rs_path).unwrap();
        if !content.contains(&mod_stmt) {
            let new_content = format!("{}{}", content, mod_stmt);
            fs::write(&final_mod_rs_path, new_content).expect("Failed to update final mod.rs");
        }
    } else {
        fs::write(&final_mod_rs_path, mod_stmt).expect("Failed to create final mod.rs");
    }

    println!(" Successfully scaffolded Handler '{}'", handler_name);
    println!("  -> {}", toml_path.display());
    println!("  -> {}", bo_path.display());
}

fn run_migrations() {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("Error: DATABASE_URL not found in environment or .env file.");
        std::process::exit(1);
    });

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        println!("🔌 LightX Migrate: Connecting to database...");
        let pool = match sqlx::mysql::MySqlPoolOptions::new().connect(&db_url).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: Failed to connect to MySQL: {}", e);
                std::process::exit(1);
            }
        };

        let migrations_dir = std::path::Path::new("migrations");
        if !migrations_dir.exists() {
            eprintln!("Error: 'migrations' directory not found in the current project.");
            std::process::exit(1);
        }

        println!("🚜 LightX Migrate: Executing pending migrations in 0(1)...");
        match sqlx::migrate::Migrator::new(migrations_dir).await {
            Ok(migrator) => {
                if let Err(e) = migrator.run(&pool).await {
                    eprintln!("Error: Migration execution failed: {}", e);
                    std::process::exit(1);
                }
                println!("✅ LightX Migrate: All migrations successfully applied.");
            }
            Err(e) => {
                eprintln!(
                    "Error: Failed to instantiate migrator (check your SQL syntax): {}",
                    e
                );
                std::process::exit(1);
            }
        }
    });
}
