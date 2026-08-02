use futures::StreamExt;
use lightx::core::AppError;
use lightx::ext::bytes::Bytes;
use lightx::ext::http_body_util::Full;
use lightx::ext::hyper::Response;

pub struct DbDemoBo;

impl DbDemoBo {
    pub async fn execute(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let pool = &ctx.mysql_pool;
        let map_err = |e: sqlx::Error| AppError::DatabaseError {
            msg: e.to_string(),
            file: file!(),
            line: line!(),
        };

        let _ = sqlx::query(include_str!("../../../migrations/mysql/0001_init.sql"))
            .execute(pool)
            .await;

        let _ = sqlx::query("DELETE FROM user_groups").execute(pool).await;
        let _ = sqlx::query("DELETE FROM user_preferences")
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM groups").execute(pool).await;
        let _ = sqlx::query("DELETE FROM users").execute(pool).await;
        let _ = sqlx::query("DELETE FROM roles").execute(pool).await;

        // --- INSERT ROLE ---
        let role = crate::MysqlRoles {
            id: 0,
            name: "Admin".to_string(),
        };
        let role_id = role.insert(pool).await.map_err(map_err)? as i32;

        // --- INSERT ---
        let user1 = crate::MysqlUsers {
            id: 0,
            role_id,
            email: "alice@daox.dev".to_string(),
            is_active: true,
        };
        let user1_id = user1.insert(pool).await.map_err(map_err)? as i32;

        // --- EXISTS ---
        let exists = crate::MysqlUsers::exists_by_id(pool, user1_id)
            .await
            .map_err(map_err)?;

        // --- GET_BY_PK ---
        let fetched = crate::MysqlUsers::get_by_id(pool, user1_id)
            .await
            .map_err(map_err)?
            .unwrap();

        // --- UPDATE_BY_PK ---
        let mut user_to_update = fetched.clone();
        user_to_update.is_active = false;
        user_to_update.update_by_id(pool).await.map_err(map_err)?;

        // --- UPSERT ---
        let mut user_to_upsert = fetched.clone();
        user_to_upsert.email = "alice_upserted@daox.dev".to_string();
        user_to_upsert.upsert(pool).await.map_err(map_err)?;

        // --- INSERT_BATCH ---
        let mut batch_users = Vec::new();
        for i in 1..=5 {
            batch_users.push(crate::MysqlUsers {
                id: 0,
                role_id,
                email: format!("user{}@daox.dev", i),
                is_active: true,
            });
        }
        crate::MysqlUsers::insert_batch(pool, &batch_users)
            .await
            .map_err(map_err)?;

        // --- COUNT ---
        let count = crate::MysqlUsers::count(pool).await.map_err(map_err)?;

        // --- LIST_PAGINATED ---
        let page = crate::MysqlUsers::list_paginated(pool, "id", 1, 3)
            .await
            .map_err(map_err)?;

        // --- LIST_BY_CURSOR ---
        let cursor_page = crate::MysqlUsers::list_by_cursor(pool, user1_id, 5)
            .await
            .map_err(map_err)?;

        // --- STREAM_ALL ---
        let mut stream_count = 0;
        {
            let mut stream = crate::MysqlUsers::stream_all(pool);
            while stream.next().await.is_some() {
                stream_count += 1;
            }
        }

        // --- COMPOSITE PK ---
        let group = crate::MysqlGroups {
            id: 0,
            name: "Superusers".to_string(),
        };
        let group_id = group.insert(pool).await.map_err(map_err)? as i32;

        let ug = crate::MysqlUserGroups {
            user_id: user1_id,
            group_id,
        };
        ug.insert(pool).await.map_err(map_err)?;

        let fetch_compos =
            crate::MysqlUserGroups::get_by_group_id_and_user_id(pool, group_id, user1_id)
                .await
                .map_err(map_err)?
                .is_some();
        crate::MysqlUserGroups::delete_by_group_id_and_user_id(pool, group_id, user1_id)
            .await
            .map_err(map_err)?;

        // --- DELETE_MANY_BY_PK ---
        let ids: Vec<i32> = vec![user1_id];
        crate::MysqlUsers::delete_many_by_id(pool, &ids)
            .await
            .map_err(map_err)?;

        // --- TRANSACTION ---
        let tx_success = {
            let mut tx = pool.begin().await.map_err(map_err)?;
            let u = crate::MysqlUsers {
                id: 0,
                role_id,
                email: "tx_rollback@daox.dev".to_string(),
                is_active: true,
            };
            u.insert(&mut *tx).await.map_err(map_err)?;
            tx.rollback().await.is_ok()
        };

        let db = "MySQL";
        let json_body = serde_json::json!({
            "status": "success",
            "database": db,
            "tests": {
                "insert_and_get": fetched.email,
                "exists": exists,
                "upsert": "success",
                "batch_insert": "success",
                "count": count,
                "paginated_results": page.len(),
                "cursor_results": cursor_page.len(),
                "streamed_results": stream_count,
                "composite_pk_works": fetch_compos,
                "transaction_rollback": tx_success
            }
        });

        let bytes = Bytes::from(json_body.to_string());
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Full::new(bytes))
            .unwrap())
    }
}
