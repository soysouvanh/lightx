use daox::DaoGenerator;
use lightx::core_generator::CoreGenerator;
use lightx::handler_generator::HandlerGenerator;
use std::env;

fn main() {
    // Load local environment variables from `.env`
    dotenvy::dotenv().ok();

    let schema_dir = env::var("LIGHTX_SCHEMA_DIR").unwrap_or_else(|_| "./schema".to_string());
    let overrides_dir =
        env::var("LIGHTX_OVERRIDES_DIR").unwrap_or_else(|_| "./overrides".to_string());
    let i18n_dir = env::var("LIGHTX_I18N_DIR").unwrap_or_else(|_| "./i18n".to_string());
    let log_dir = env::var("LIGHTX_LOG_DIR").unwrap_or_else(|_| "./log".to_string());

    std::fs::create_dir_all(&schema_dir).unwrap_or_default();
    std::fs::create_dir_all(&overrides_dir).unwrap_or_default();
    std::fs::create_dir_all(&i18n_dir).unwrap_or_default();
    std::fs::create_dir_all(&log_dir).unwrap_or_default();

    scaffold_readmes(&schema_dir, &overrides_dir, &i18n_dir, &log_dir);

    println!("cargo:rerun-if-changed={}", schema_dir);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DATABASE_URL");
    println!("cargo:rerun-if-env-changed=LIGHTX_SCHEMA_DIR");
    println!("cargo:rerun-if-env-changed=LIGHTX_OVERRIDES_DIR");
    println!("cargo:rerun-if-env-changed=LIGHTX_I18N_DIR");
    println!("cargo:rerun-if-env-changed=LIGHTX_LOG_DIR");
    println!("cargo:rerun-if-env-changed=LIGHTX_SCHEMA_README");
    println!("cargo:rerun-if-env-changed=LIGHTX_OVERRIDES_README");
    println!("cargo:rerun-if-env-changed=LIGHTX_I18N_README");
    println!("cargo:rerun-if-env-changed=LIGHTX_LOG_README");

    // The isolated target directory injected by Cargo (absolute isolation of generated code)
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable is not set");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let daox_out_dir = std::path::PathBuf::from(manifest_dir)
        .join("src")
        .join("daox_generated");
    std::fs::create_dir_all(&daox_out_dir).unwrap_or_default();

    // Instantiation of the "Shift-Left" generation framework
    let generator = DaoGenerator::new(&schema_dir, daox_out_dir.to_str().unwrap());

    // =====================================================================
    // 1. DATABASE-FIRST: Introspection and TOML Dictionary Generation
    // =====================================================================
    // We simulate the `daox` behavior: read the live Database and generate
    // the configuration files into the `schema/` folder.
    if let Ok(db_url) = env::var("DATABASE_URL") {
        // `build.rs` is inherently synchronous, so we instantiate a Tokio runtime
        // to handle the async introspection.
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            generator
                .introspect(&db_url)
                .await
                .expect("Failed to introspect MySQL database");
        });
    } else {
        println!(
            "cargo:warning= DATABASE_URL not set in environment. Skipping database introspection."
        );
    }

    // =====================================================================
    // 2. METAPROGRAMMING PIPELINE: Rust DAO Generation
    // =====================================================================
    // Parses the TOML dictionary and writes `lightx_dao_generated.rs` into `OUT_DIR`.
    generator
        .generate_dao()
        .expect("Failed to generate LightX DAO code");

    // =====================================================================
    // 3. AOP PIPELINE: Handlers Generation
    // =====================================================================
    // Parses the `handlers/*.toml` files and generates `lightx_handlers_generated.rs`.
    println!("cargo:rerun-if-changed=handlers/");
    let handler_generator = HandlerGenerator::new("./handlers", &out_dir);
    handler_generator
        .generate_handlers()
        .expect("Failed to generate LightX Handlers code");

    // =====================================================================
    // 4. CORE PIPELINE: Static Router Generation
    // =====================================================================
    let core_generator = CoreGenerator::new("./handlers", &i18n_dir, &out_dir);
    core_generator
        .generate_core()
        .expect("Failed to generate LightX Core code");
}

fn scaffold_readmes(schema: &str, overrides: &str, i18n: &str, log: &str) {
    if std::env::var("LIGHTX_OVERRIDES_README").unwrap_or_else(|_| "true".to_string()) == "true" {
        let _ = std::fs::write(
            std::path::Path::new(overrides).join("README.md"),
            r###"#  LightX Overrides Strategy

Welcome to the overrides directory!

LightX's "Database-First" philosophy completely overwrites the `schema/` directory on every compilation. 
**Never modify the files in `schema/` as they will be overwritten!**

If you want to override business validation rules (e.g., forcing `min_length = 5` on an SQL column) or inject custom metadata, you must reproduce the exact table topology here.

##  Example
To override the `last_name` column of the `users` table:
1. Create a `users/` directory within this folder.
2. Create a `users/last_name.toml` file here strictly containing ONLY the values you wish to mutate.

```toml
[min_length]
value = 5
message = "Last name must be at least 5 characters long (Manual override!)"
```"###,
        );

        let _ = std::fs::write(
            std::path::Path::new(overrides).join("README.fr.md"),
            r###"#  Stratégie des Overrides LightX

Bienvenue dans le répertoire des surcharges (overrides) !

La philosophie "Database-First" de LightX écrase entièrement le dossier `schema/` à chaque cycle de compilation. 
**Ne modifiez jamais manuellement les fichiers dans `schema/` car toutes vos modifications seront détruites !**

Si vous souhaitez surcharger des règles de validation métier (ex: forcer `min_length = 5` sur une colonne SQL) ou lier de nouvelles métadonnées, vous devez reproduire l'arborescence exacte de la table base de données ici.

##  Exemple
Pour créer une redéfinition sur la colonne `last_name` de la table `users` :
1. Créez un répertoire `users/` ici.
2. Créez ensuite le fichier `users/last_name.toml` ici avec UNIQUEMENT les valeurs à écraser.

```toml
[min_length]
value = 5
message = "Le nom de famille doit faire au moins 5 caractères (Surcharge manuelle !)"
```"###,
        );
    }

    if std::env::var("LIGHTX_SCHEMA_README").unwrap_or_else(|_| "true".to_string()) == "true" {
        let _ = std::fs::write(
            std::path::Path::new(schema).join("README.md"),
            r###"#  LightX Single Source of Truth

Welcome to the schema architecture directory!

This directory is strictly **auto-generated** at compile time by the LightX framework through deep database introspection. It acts as the absolute Data Dictionary (Single Source of Truth) for the entire application ecosystem.

##  CRITICAL WARNING
**DO NOT MANUALLY EDIT ANY FILES IN THIS DIRECTORY!**

All structural files (`.toml`) here are regenerated from scratch during the `cargo build` macro expansion phase. Any manual file modifications or additions will be permanently overwritten.

If you need to force custom validation rules or adjust generated types, you must utilize the `../overrides/` directory instead."###,
        );

        let _ = std::fs::write(
            std::path::Path::new(schema).join("README.fr.md"),
            r###"#  Architecture Source de Vérité Unique (LightX)

Bienvenue dans le répertoire du modèle de données pur !

Ce dossier est **généré automatiquement** à la compilation par le framework LightX via une introspection avancée de votre base de données. Il agit comme le dictionnaire de données absolu (Single Source of Truth) de tout le cycle de vie de l'application.

##  AVERTISSEMENT CRITIQUE
**NE MODIFIEZ MANUELLEMENT AUCUN FICHIER DANS CE RÉPERTOIRE !**

Tous les fichiers de définition de structure (`.toml`) ici sont détruits puis regénérés de zéro à chaque appel de `cargo build`. Toute modification humaine sera définitivement et formellement perdue.

Si vous avez besoin de forcer des règles de validation sur mesure ou de corriger un type dégénéré, vous devez utiliser impérativement le dossier `../overrides/` à la place."###,
        );
    }

    if std::env::var("LIGHTX_I18N_README").unwrap_or_else(|_| "true".to_string()) == "true" {
        let _ = std::fs::write(
            std::path::Path::new(i18n).join("README.md"),
            r###"#  LightX Internationalization (i18n)

Welcome to the Zero-Overhead translation directory!

This architectural repository stores static TOML translation pipelines precisely mapped to the underlying system constraints and business logic components.

##  Scale-Resistant Structure
To guarantee limitless horizontal scalability without Git "merge hells", this namespace aggressively mimics the database topological layout (allocating strictly one translation definition per column/handler).

- `schema.toml`: Global fallback error structures injected during compilation (e.g. integer bounds checking).
- `handlers/`: A human-defined dictionary exclusively for mapping backend application flow failures (e.g., `not_found`).
- `overrides/`: Exact matching namespace resolving highly specific backend custom rules to localized strings.

In this architecture, language logic is offloaded entirely to the Frontend. The backend solely resolves static translation keys in `O(1)` runtime operations."###,
        );

        let _ = std::fs::write(
            std::path::Path::new(i18n).join("README.fr.md"),
            r###"#  Internationalisation LightX (i18n)

Bienvenue dans le répertoire des traductions sans surcoût d'exécution (Zero-Overhead) !

Ce répertoire est l'ancre stockant les canalisations de vos messages d'erreurs en format TOML pur, toutes strictement couplées avec les contraintes d'intégrité de la base ou l'implémentation métier.

##  Résistance à l'Hyper-Croissance (Scalabilité)
Pour garantir une stabilité absolue et éviter les conflits Git infernaux sur des projets de milliers de tables, cet espace singe farouchement la topologie granulaire de la base de données.

- `schema.toml`: Le modèle de traduction global pour toute erreur générique liée aux types.
- `handlers/`: Lexique défini par les développeurs pour transcrire des ruptures logiques complexes.
- `overrides/`: Miroir exact identifiant les règles métiers sur-mesure pour être injectées au client.

L'objectif ultime est le déport absolu : le moteur rust du serveur Backend passe son temps à manipuler de la clé statique en `O(1)`. Libre ensuite au Frontend d'enrichir le côté visuel (React, Vue, ou autre)."###,
        );
    }

    if std::env::var("LIGHTX_LOG_README").unwrap_or_else(|_| "true".to_string()) == "true" {
        let _ = std::fs::write(
            std::path::Path::new(log).join("README.md"),
            r###"#  LightX Application Logs

Welcome to the local development logs directory!

This directory captures critical asynchronous runtime telemetry and asynchronous access anomalies automatically dumped by the embedded LightX Core router mechanism (Hyper engine).

##  Environment Behaviors
- In this development repository (`lightx-test`), logs materialize locally corresponding to the `LIGHTX_LOG_DIR` environment definition within `.env`.
- In a true production tier, execution dictates that `LIGHTX_LOG_DIR` be natively wired into a centralized pipeline, such as `/var/log/lightx`, allowing remote log aggregators (e.g. Datadog, ELK) seamless streaming capabilities.

Double check internally that this directory path gracefully ignores version control (`.gitignore`) to avoid enormous payloads entering the repository history."###,
        );

        let _ = std::fs::write(
            std::path::Path::new(log).join("README.fr.md"),
            r###"#  Journaux Applicatifs (Logs) LightX

Bienvenue dans le registre de développement local !

Ce répertoire capture la télémétrie structurée et les traces asynchrones critiques vomies de manière ininterrompue par l'intelligence du routeur de LightX Core.

##  Comportement Contextuel
- Dans le contexte de test lié à ce dépôt, ces traces atterrissent ici localement selon la variable `LIGHTX_LOG_DIR` de votre point d'ancrage `.env`.
- En contexte de production d'entreprise stricte, la variable `LIGHTX_LOG_DIR` est nativement poussée sur `/var/log/lightx` (ou directement redirigée en Stdout console) afin de laisser des monstres comme ELK ou Datadog capter les métriques.

Par nature, assurez vous explicitement que ce dernier dossier figure dans vos règles d'exclusion `.gitignore` pour ne pas noyer l'historique du projet."###,
        );
    }
}
