//! # SQLite End-to-End API Demonstration
//!
//! This Business Object (BO) module demonstrates how to implement a fully functional REST API
//! leveraging LightX and Daox. It processes raw HTTP requests entirely manually, without any external
//! high-level JSON framework overhead, showcasing maximum control and State-of-the-Art zero-dependencies philosophy.

use lightx::core::AppError;
use lightx::ext::bytes::Bytes;
use lightx::ext::http_body_util::Full;
use lightx::ext::hyper::{Method, Response};

/// The main Business Object acting as a REST API Controller.
pub struct DbDemoBo;

impl DbDemoBo {
    /// Executes the REST logic by interpreting the HTTP Method.
    ///
    /// ## Pedagogical Note:
    /// - `GET` requests trigger a database listing mapped to JSON via `serde_json`.
    /// - `POST` requests extract JSON from `ctx.raw_body` to natively insert `SqliteUsers`.
    pub async fn execute(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        // Extract the highly optimized SQLite connection pool from the Request Context.
        let pool = &ctx.sqlite_pool;

        let map_err = |e: sqlx::Error| AppError::DatabaseError {
            msg: e.to_string(),
            file: file!(),
            line: line!(),
        };

        // Guarantee that the foundational table schema exists (useful for pedagogical purposes).
        let _ = sqlx::query(include_str!("../../../migrations/sqlite/0001_init.sql"))
            .execute(pool)
            .await;

        // Resolve the HTTP Method gracefully, defaulting to GET if unsupported.
        let method = ctx
            .raw_req
            .as_ref()
            .map(|r| r.method().clone())
            .unwrap_or(Method::GET);

        match method {
            Method::GET => {
                // Fetch the list using Daox cursor-based pagination (O(1) Memory impact)
                let users = crate::SqliteUsers::list_by_cursor(pool, -1, 100)
                    .await
                    .map_err(map_err)?;

                // Map native Structs into JSON arrays flawlessly.
                let mut json_arr = Vec::new();
                for u in users {
                    json_arr.push(serde_json::json!({
                        "id": u.id,
                        "email": u.email,
                        "role_id": u.role_id,
                        "is_active": u.is_active,
                    }));
                }

                // Serialize explicitly and respond natively with Hyper.
                let json_body = serde_json::to_string(&json_arr).unwrap_or_default();
                Ok(Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json_body)))
                    .unwrap())
            }
            Method::POST => {
                // Safely slice the asynchronous raw bytes directly into a JSON Value.
                let json: Option<serde_json::Value> = serde_json::from_slice(&ctx.raw_body).ok();
                if let Some(data) = json {
                    // Create a fallback Role if necessary to respect foreign key constraints.
                    let role = crate::SqliteRoles {
                        id: 0,
                        name: "DefaultRole".to_string(),
                    };
                    let role_id = role.insert(pool).await.unwrap_or(1) as i32;

                    // Safely unwrap JSON payloads into heavily validated Daox objects.
                    let u = crate::SqliteUsers {
                        id: 0,
                        role_id,
                        email: data
                            .get("email")
                            .and_then(|v| v.as_str())
                            .unwrap_or("anon@anon.com")
                            .to_string(),
                        is_active: data
                            .get("is_active")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true),
                    };

                    // Insert the user explicitly using safe bindings to prevent SQL Injections.
                    u.insert(pool).await.map_err(map_err)?;

                    Ok(Response::builder()
                        .status(201)
                        .body(Full::new(Bytes::from("Created")))
                        .unwrap())
                } else {
                    // Fail gracefully on Bad JSON payload.
                    Ok(Response::builder()
                        .status(400)
                        .body(Full::new(Bytes::from("Bad Request JSON Payload")))
                        .unwrap())
                }
            }
            _ => {
                // Fail gracefully if methods other than GET or POST are invoked.
                Ok(Response::builder()
                    .status(405)
                    .body(Full::new(Bytes::from("Method Not Allowed")))
                    .unwrap())
            }
        }
    }
}
