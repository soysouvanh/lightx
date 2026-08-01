use lightx::core::AppError;
use lightx::ext::bytes::Bytes;
use lightx::ext::http_body_util::Full;
use lightx::ext::hyper::Response;

pub struct DbDemoBo;

impl DbDemoBo {
    pub async fn execute(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let _ = sqlx::query(include_str!("../../../migrations/sqlite/0001_init.sql"))
            .execute(&ctx.sqlite_pool)
            .await;

        let role = crate::SqliteRoles {
            id: 0,
            name: "Admin SQLite".to_string(),
        };
        let role_id = if let Some(tx) = ctx.sqlite_tx.as_mut() {
            role.insert(&mut **tx).await
        } else {
            role.insert(&ctx.sqlite_pool).await
        }
        .map_err(|e| AppError::DatabaseError {
            msg: e.to_string(),
            file: file!(),
            line: line!(),
        })?;

        let user = crate::SqliteUsers {
            id: 0,
            role_id: role_id as i32,
            email: "sqlite@lightx.dev".to_string(),
            is_active: true,
        };
        let user_id = if let Some(tx) = ctx.sqlite_tx.as_mut() {
            user.insert(&mut **tx).await
        } else {
            user.insert(&ctx.sqlite_pool).await
        }
        .map_err(|e| AppError::DatabaseError {
            msg: e.to_string(),
            file: file!(),
            line: line!(),
        })?;

        let fetched = if let Some(tx) = ctx.sqlite_tx.as_mut() {
            crate::SqliteUsers::get_by_id(&mut **tx, user_id as i32).await
        } else {
            crate::SqliteUsers::get_by_id(&ctx.sqlite_pool, user_id as i32).await
        }
        .map_err(|e| AppError::DatabaseError {
            msg: e.to_string(),
            file: file!(),
            line: line!(),
        })?;

        Self::build_json_response("SQLite", fetched.map(|u| u.email))
    }

    fn build_json_response(
        db: &str,
        user_email: Option<String>,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let json_body = serde_json::json!({ "status": "success", "database": db, "fetched_user_email": user_email });
        let bytes = Bytes::from(json_body.to_string());
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Full::new(bytes))
            .unwrap())
    }
}
