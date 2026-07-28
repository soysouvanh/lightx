///  `AppError`: The Global Domain Error Enumeration.
///
/// In LightX, all internal layers (Handlers, BO, DAO) bubble their failures up
/// using a unified error type. This enforces strict error management and prevents
/// the dreaded `panic!()` in production.
///
/// By centralizing errors, the web server can easily implement a global error
/// handler to automatically translate these variants into proper HTTP responses:
/// - `ParameterError` -> `400 Bad Request`
/// - `BusinessError` -> `422 Unprocessable Entity`
/// - `AuthenticationError` -> `401 Unauthorized`
/// - `PermissionError` -> `403 Forbidden`
/// - `DatabaseError` -> `500 Internal Server Error`
/// - `SystemError` -> `500 Internal Server Error`
///
/// # Examples
///
/// ```
/// use lightx::core::AppError;
///
/// let err = AppError::RouteNotFound;
/// assert_eq!(err.to_string(), "Route Not Found");
/// ```
#[derive(Debug)]
pub enum AppError {
    /// 401 Unauthorized
    AuthenticationError {
        msg: String,
        file: &'static str,
        line: u32,
    },

    /// 403 Forbidden
    PermissionError {
        msg: String,
        file: &'static str,
        line: u32,
    },

    /// 400 Bad Request
    ParameterError {
        field: String,
        msg: String,
        file: &'static str,
        line: u32,
    },

    /// 422 Unprocessable Entity (Business Rules)
    BusinessError {
        field: String,
        msg: String,
        file: &'static str,
        line: u32,
    },

    /// 500 Internal Server Error (Database)
    DatabaseError {
        msg: String,
        file: &'static str,
        line: u32,
    },

    /// 500 Internal Server Error (System)
    SystemError {
        msg: String,
        file: &'static str,
        line: u32,
    },

    /// 429 Too Many Requests (Rate Limit)
    TooManyRequests,

    /// 404 Not Found (Route)
    RouteNotFound,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::AuthenticationError { msg, file, line } => {
                write!(f, "Authentication Error at {}:{}: {}", file, line, msg)
            }
            AppError::PermissionError { msg, file, line } => {
                write!(f, "Permission Error at {}:{}: {}", file, line, msg)
            }
            AppError::ParameterError {
                field,
                msg,
                file,
                line,
            } => write!(
                f,
                "Parameter Error on '{}' at {}:{}: {}",
                field, file, line, msg
            ),
            AppError::BusinessError {
                field,
                msg,
                file,
                line,
            } => {
                write!(
                    f,
                    "Business Error on '{}' at {}:{}: {}",
                    field, file, line, msg
                )
            }
            AppError::DatabaseError { msg, file, line } => {
                write!(f, "Database Error at {}:{}: {}", file, line, msg)
            }
            AppError::SystemError { msg, file, line } => {
                write!(f, "System Error at {}:{}: {}", file, line, msg)
            }
            AppError::TooManyRequests => write!(f, "Too Many Requests"),
            AppError::RouteNotFound => write!(f, "Route Not Found"),
        }
    }
}

impl std::error::Error for AppError {}

#[diagnostic::on_unimplemented(
    message = "Le Business Object doit renvoyer un type implémentant `IntoLightXResponse`",
    label = "Type de retour non reconnu pour le routeur LightX",
    note = "En principe, votre Business Object devrait retourner un `Result<T, AppError>` où T implémente IntoLightXResponse."
)]
/// Définit le contrat strict de restitution d'une réponse réseau depuis un Business Object.
/// Ce trait est le socle de la pédagogie Rust (DX) via des erreurs de compilations sur-mesure.
///
/// # Examples
///
/// ```
/// use lightx::core::{IntoLightXResponse, AppError};
///
/// struct MyResponse;
/// impl IntoLightXResponse for MyResponse {
///     fn into_response(self) -> Result<String, AppError> {
///         Ok("{}".to_string())
///     }
/// }
/// ```
pub trait IntoLightXResponse {
    fn into_response(self) -> Result<String, AppError>;
}

impl IntoLightXResponse for String {
    fn into_response(self) -> Result<String, AppError> {
        Ok(self)
    }
}

// NOTE: The `RequestContext` struct is generated at compile time by `build.rs`
// into the `OUT_DIR` because its fields depend strictly on the `databases.toml` schema configuration.
// The `TypeMap` has been removed from the framework (Phase 0.3) — all typed data
// is now passed as explicit, typed parameters through function signatures.

#[macro_export]
/// Interrompt le flux et retourne une `BusinessError` (HTTP 422).
///
/// # Examples
///
/// ```
/// use lightx::bail_business_rule;
/// use lightx::core::AppError;
///
/// fn check_business() -> Result<(), AppError> {
///     bail_business_rule!("email", "Format d'email invalide.");
///     Ok(())
/// }
/// ```
macro_rules! bail_business_rule {
    ($field:expr, $msg:expr) => {
        return Err(lightx::core::AppError::BusinessError {
            field: $field.into(),
            msg: $msg.into(),
            file: file!(),
            line: line!(),
        })
    };
}

#[macro_export]
/// Interrompt le flux et retourne une `AuthenticationError` (HTTP 401).
///
/// # Examples
///
/// ```
/// use lightx::bail_authentication;
/// use lightx::core::AppError;
///
/// fn check_auth() -> Result<(), AppError> {
///     bail_authentication!("Token expiré ou invalide.");
///     Ok(())
/// }
/// ```
macro_rules! bail_authentication {
    ($msg:expr) => {
        return Err(lightx::core::AppError::AuthenticationError {
            msg: $msg.into(),
            file: file!(),
            line: line!(),
        })
    };
}

#[macro_export]
/// Interrompt le flux et retourne une `PermissionError` (HTTP 403).
///
/// # Examples
///
/// ```
/// use lightx::bail_permission;
/// use lightx::core::AppError;
///
/// fn check_permission() -> Result<(), AppError> {
///     bail_permission!("Vous n'avez pas les droits d'administration.");
///     Ok(())
/// }
/// ```
macro_rules! bail_permission {
    ($msg:expr) => {
        return Err(lightx::core::AppError::PermissionError {
            msg: $msg.into(),
            file: file!(),
            line: line!(),
        })
    };
}

#[macro_export]
/// Interrompt le flux et retourne une `SystemError` (HTTP 500).
///
/// # Examples
///
/// ```
/// use lightx::bail_system;
/// use lightx::core::AppError;
///
/// fn check_system() -> Result<(), AppError> {
///     bail_system!("Erreur matérielle inattendue.");
///     Ok(())
/// }
/// ```
macro_rules! bail_system {
    ($msg:expr) => {
        return Err(lightx::core::AppError::SystemError {
            msg: $msg.into(),
            file: file!(),
            line: line!(),
        })
    };
}

static JWT_DECODING_KEY: std::sync::OnceLock<crate::ext::jsonwebtoken::DecodingKey> =
    std::sync::OnceLock::new();
static JWT_VALIDATION: std::sync::OnceLock<crate::ext::jsonwebtoken::Validation> =
    std::sync::OnceLock::new();

#[derive(serde::Deserialize)]
struct JwtClaims {
    sub: Option<String>,
    id: Option<String>,
}

/// Global Authenticator Primitive (AOP)
pub async fn verify_jwt(token: &str) -> Result<String, AppError> {
    let key = JWT_DECODING_KEY.get_or_init(|| {
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "military_grade_secret_default_change_me_in_prod".to_string());
        crate::ext::jsonwebtoken::DecodingKey::from_secret(secret.as_bytes())
    });

    let validation = JWT_VALIDATION.get_or_init(|| {
        let mut v =
            crate::ext::jsonwebtoken::Validation::new(crate::ext::jsonwebtoken::Algorithm::HS256);
        // By default, jsonwebtoken enforces strict expiration verification if 'exp' claim is present
        v.validate_exp = true;
        v
    });

    // Zero-overhead mathematical execution (HS256 is sub-microsecond, no context-switch required)
    match crate::ext::jsonwebtoken::decode::<JwtClaims>(token, key, validation) {
        Ok(token_data) => {
            let claims = token_data.claims;
            if let Some(sub) = claims.sub {
                Ok(sub)
            } else if let Some(id) = claims.id {
                Ok(id)
            } else {
                Err(AppError::AuthenticationError {
                    msg: "JWT does not contain 'sub' or 'id' claim".into(),
                    file: file!(),
                    line: line!(),
                })
            }
        }
        Err(e) => Err(AppError::AuthenticationError {
            msg: format!("Invalid JWT: {}", e),
            file: file!(),
            line: line!(),
        }),
    }
}
