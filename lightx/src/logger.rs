use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::mpsc::{self, SyncSender};
use std::thread;

/// Represents the severity level of a log message.
#[derive(Clone, Copy)]
pub enum LogLevel {
    /// Severe errors that may cause the application to fail.
    Error,
    /// Non-critical issues that should be monitored.
    Warning,
    /// General informational messages about the application's state.
    Info,
    /// Detailed debugging information, typically useful during development.
    Debug,
    /// Used for business tracing and raw data export (e.g., ElasticSearch).
    /// Produces a `.json` file without date/level prefixes, ideal for NDJSON processing.
    Audit,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warning => "warning",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Audit => "audit",
        }
    }
}

pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
}

static LOGGER_TX: OnceLock<SyncSender<LogMessage>> = OnceLock::new();
/// Initializes the asynchronous, O(1) logging manager.
///
/// This function launches a dedicated OS thread in the background to ensure that HTTP
/// requests are never blocked by disk I/O operations. It follows the "Fail-Fast" principle:
/// if the specified `root_path` directory cannot be created or accessed, it will immediately
/// return a synchronous error.
///
/// # Examples
///
/// ```no_run
/// use lightx::logger;
///
/// // Initialize the logger to write files into the "./logs" directory.
/// logger::init("./logs").expect("Failed to initialize the logger directory");
/// ```
///
/// # Errors
///
/// Returns an [`std::io::Result`] which will be an `Err` if the directory is inaccessible or
/// lacks write permissions.
pub fn init(root_path: &str) -> std::io::Result<()> {
    // Capacité fixée à 10_000 pour éviter les vulnérabilités de type OOM par épuisement mémoire (DoS)
    let (tx, rx) = mpsc::sync_channel::<LogMessage>(10_000);

    // On conserve le chemin racine en propre
    let root = PathBuf::from(root_path);

    // Fail-fast : Vérification immédiate et synchrone des permissions
    fs::create_dir_all(&root)?;

    // Le Thread O(1) de journalisation
    thread::spawn(move || {
        while let Ok(log_msg) = rx.recv() {
            let now = Local::now();
            let date_str = now.format("%Y-%m-%d").to_string();
            let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

            let mut dir_path = root.clone();
            dir_path.push(&date_str);

            // Création paresseuse du dossier
            if let Err(e) = fs::create_dir_all(&dir_path) {
                eprintln!(
                    " [LOGGER CRITICAL] Impossible de créer le dossier de log : {:?}",
                    e
                );
                continue;
            }

            let (file_name, log_line) = match log_msg.level {
                LogLevel::Audit => (
                    format!("{}-{}.json", log_msg.level.as_str(), date_str),
                    format!("{}\n", log_msg.message), // Pas de préfixe pour respecter le format NDJSON
                ),
                _ => (
                    format!("{}-{}.txt", log_msg.level.as_str(), date_str),
                    format!(
                        "[{}] [{}] {}\n",
                        time_str,
                        log_msg.level.as_str().to_uppercase(),
                        log_msg.message
                    ),
                ),
            };

            let mut file_path = dir_path;
            file_path.push(&file_name);

            let mut file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!(
                        " [LOGGER CRITICAL] Impossible d'ouvrir le fichier de log : {:?}",
                        e
                    );
                    continue;
                }
            };

            if let Err(e) = file.write_all(log_line.as_bytes()) {
                eprintln!(
                    " [LOGGER CRITICAL] Impossible d'écrire dans le fichier de log : {:?}",
                    e
                );
            }
        }
    });

    LOGGER_TX.set(tx).unwrap_or_else(|_| {
        eprintln!(" [LOGGER WARNING] Le logger a déjà été initialisé.");
    });

    Ok(())
}

/// Sends a message to the logger in O(1) memory time without blocking.
///
/// If the logger hasn't been initialized via [`init`], this function will safely fallback
/// by printing the message directly to standard error (`stderr`).
///
/// # Examples
///
/// ```
/// use lightx::logger::{self, LogLevel};
///
/// logger::write(LogLevel::Info, "Manual log entry".to_string());
/// ```
pub fn write(level: LogLevel, message: String) {
    if let Some(tx) = LOGGER_TX.get() {
        // Politique de drop (abandon) : on utilise `try_send` car il ne bloque pas l'exécution du thread HTTP.
        // Si la file d'attente disque sature (Backpressure), les logs excédentaires sont jetés pour survivre.
        let _ = tx.try_send(LogMessage { level, message });
    } else {
        // Repli par défaut sur la console si non configuré
        eprintln!("[{}] {}", level.as_str().to_uppercase(), message);
    }
}

/// Shortcut to log an informational message ([`LogLevel::Info`]).
///
/// # Examples
///
/// ```
/// use lightx::logger;
///
/// logger::info("Server started successfully.".to_string());
/// ```
pub fn info(message: String) {
    write(LogLevel::Info, message);
}

/// Shortcut to log an error message ([`LogLevel::Error`]).
///
/// # Examples
///
/// ```
/// use lightx::logger;
///
/// logger::error("Failed to connect to the database.".to_string());
/// ```
pub fn error(message: String) {
    write(LogLevel::Error, message);
}

/// Shortcut to log a warning message ([`LogLevel::Warning`]).
///
/// # Examples
///
/// ```
/// use lightx::logger;
///
/// logger::warning("Disk space is running low.".to_string());
/// ```
pub fn warning(message: String) {
    write(LogLevel::Warning, message);
}

/// Shortcut to log debugging information ([`LogLevel::Debug`]).
///
/// # Examples
///
/// ```
/// use lightx::logger;
///
/// logger::debug(format!("Current thread id: {:?}", std::thread::current().id()));
/// ```
pub fn debug(message: String) {
    write(LogLevel::Debug, message);
}

/// Shortcut to log a raw business trace without formatting ([`LogLevel::Audit`]).
///
/// This is specifically designed for writing raw JSON payloads that can be natively
/// ingested by tools like ElasticSearch or Filebeat without any Grok parsing.
///
/// # Examples
///
/// ```
/// use lightx::logger;
///
/// let payload = r#"{"event": "admin_creation", "user_id": 42}"#.to_string();
/// logger::audit(payload);
/// ```
pub fn audit(message: String) {
    write(LogLevel::Audit, message);
}

/// Initializes natively the `tracing` framework and its OpenTelemetry OTLP channels.
///
/// Ensures telemetry traces (including SQLx quantum queries and business logs) are harmoniously
/// pushed to Jaeger/Prometheus in background without overhead.
pub fn init_telemetry(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .build()?;

        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_service_name(service_name.to_string())
                    .build(),
            )
            .build();

        opentelemetry::global::set_tracer_provider(provider);
        opentelemetry::global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );
        let tracer = opentelemetry::global::tracer(service_name.to_string());

        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(telemetry)
            .try_init()?;
    } else {
        // Zéro-log boilerplate: activate local terminal tracing if Jaeger isn't available
        let _ = tracing_subscriber::fmt().try_init();
    }
    Ok(())
}
