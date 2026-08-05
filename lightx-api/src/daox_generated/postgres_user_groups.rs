#[cfg(not(feature = "validation"))]
compile_error!(
    "Daox: the 'validation' feature is disabled. \
     Format regex checks in validate() will be skipped. \
     Enable with: features = [\"validation\"]"
);
#[allow(clippy::all)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PostgresUserGroups {
    pub group_id: i32,
    pub user_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostgresUserGroupsOrderBy {
    GroupIdAsc,
    GroupIdDesc,
    UserIdAsc,
    UserIdDesc,
}

impl PostgresUserGroupsOrderBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            PostgresUserGroupsOrderBy::GroupIdAsc => r#""group_id" ASC"#,
            PostgresUserGroupsOrderBy::GroupIdDesc => r#""group_id" DESC"#,
            PostgresUserGroupsOrderBy::UserIdAsc => r#""user_id" ASC"#,
            PostgresUserGroupsOrderBy::UserIdDesc => r#""user_id" DESC"#,
        }
    }
}

#[allow(clippy::all)]
impl PostgresUserGroups {
    #[allow(unused_comparisons, unused_mut)]
    pub fn validate(&self) -> Result<(), Vec<String>> {
        #[cfg(not(feature = "validation"))]
        {
            // Formats validation is disabled
        }
        let mut errors = Vec::new();
        if let Some(v) = Some(&self.group_id) {
            if (*v as i128) < (0 as i128) {
                errors.push("group_id: minimum value '0' not met".into());
            }
        }
        if let Some(v) = Some(&self.group_id) {
            if (*v as i128) > (2147483647 as i128) {
                errors.push("group_id: maximum value '2147483647' exceeded".into());
            }
        }
        if let Some(v) = Some(&self.user_id) {
            if (*v as i128) < (0 as i128) {
                errors.push("user_id: minimum value '0' not met".into());
            }
        }
        if let Some(v) = Some(&self.user_id) {
            if (*v as i128) > (2147483647 as i128) {
                errors.push("user_id: maximum value '2147483647' exceeded".into());
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Returns the total number of rows in the table.
    ///
    /// **⚠️ Performance Warning:** On some databases (e.g., MySQL/InnoDB, PostgreSQL),
    /// a `COUNT(*)` without a `WHERE` clause can cause a full table scan,
    /// which may take a long time on large tables (e.g. >10M rows).
    /// Consider caching this value or using an approximate row count from
    /// `information_schema.tables` or `pg_class` if exact precision is not required.
    #[deprecated(
        since = "0.2.0",
        note = "Use `approximate_count` instead to prevent full table scans."
    )]
    pub async fn count<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"SELECT COUNT(*) FROM "user_groups""#;
        let (count,): (i64,) = sqlx::query_as(query).fetch_one(executor).await?;
        Ok(count as u64)
    }

    /// Returns an approximate total number of rows in the table using database statistics (O(1)).
    /// This is extremely fast for huge tables but the number may be slightly outdated until the next VACUUM/ANALYZE.
    pub async fn approximate_count<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"SELECT reltuples::bigint FROM pg_class WHERE relname = $1"#;
        let count: Option<(i64,)> = sqlx::query_as(query)
            .bind("user_groups")
            .fetch_optional(executor)
            .await?;
        Ok(count.map(|(c,)| c.max(0) as u64).unwrap_or(0))
    }

    /// Streams rows from the table, ordered by the primary key.
    /// **⚠️ Performance Warning:** Streaming a whole table without a limit or timeout can cause connection pool starvation.
    /// A `limit` parameter is now mandatory to prevent Unbounded Streaming DoS. Timeouts are managed by the underlying sqlx `AnyPoolOptions` settings.
    #[deprecated(
        since = "0.2.0",
        note = "Use cursor-based pagination instead to prevent pool starvation."
    )]
    pub fn stream_all<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres> + 'e>(
        executor: E,
        limit: i64,
    ) -> impl futures::Stream<Item = sqlx::Result<Self>> + 'e {
        let limit = limit.clamp(1, 10000);
        let query = r#"SELECT "group_id", "user_id" FROM "user_groups" ORDER BY "group_id" ASC, "user_id" ASC LIMIT $1"#;
        sqlx::query_as::<_, Self>(query).bind(limit).fetch(executor)
    }

    pub async fn get_by_group_id_and_user_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    >(
        executor: E,
        group_id: i32,
        user_id: i32,
    ) -> sqlx::Result<Option<Self>> {
        let query = r#"SELECT "group_id", "user_id" FROM "user_groups" WHERE "group_id" = $1 AND "user_id" = $2"#;
        sqlx::query_as::<_, Self>(query)
            .bind(group_id)
            .bind(user_id)
            .fetch_optional(executor)
            .await
    }

    pub async fn exists_by_group_id_and_user_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    >(
        executor: E,
        group_id: i32,
        user_id: i32,
    ) -> sqlx::Result<bool> {
        let query =
            r#"SELECT 1 FROM "user_groups" WHERE "group_id" = $1 AND "user_id" = $2 LIMIT 1"#;
        let exists: Option<(i32,)> = sqlx::query_as(query)
            .bind(group_id)
            .bind(user_id)
            .fetch_optional(executor)
            .await?;
        Ok(exists.is_some())
    }

    pub async fn insert_unchecked<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"INSERT INTO "user_groups" ("group_id", "user_id") VALUES ($1, $2)"#;
        let result = sqlx::query::<sqlx::Postgres>(query)
            .bind(&self.group_id)
            .bind(&self.user_id)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn insert<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        if let Err(e) = self.validate() {
            return Err(sqlx::Error::Protocol(e.join(", ").into()));
        }
        self.insert_unchecked(executor).await
    }

    /// Inserts a batch of records using Postgres COPY (ultra-fast).
    /// WARNING: To guarantee atomicity across all chunks, you MUST pass an explicit `sqlx::Transaction` as the `executor`.
    pub async fn insert_batch<'e>(
        executor: &mut sqlx::Transaction<'e, sqlx::Postgres>,
        items: &[Self],
    ) -> sqlx::Result<u64> {
        if items.is_empty() {
            return Ok(0);
        }
        for (idx, item) in items.iter().enumerate() {
            if let Err(e) = item.validate() {
                return Err(sqlx::Error::Protocol(
                    format!(
                        "insert_batch: item {} failed validation: {}",
                        idx,
                        e.join(", ")
                    )
                    .into(),
                ));
            }
        }
        let mut copy_in = executor
            .copy_in_raw(
                r#"COPY "user_groups" ("group_id", "user_id") FROM STDIN WITH (FORMAT csv)"#,
            )
            .await?;
        for chunk in items.chunks(1000) {
            let est: usize = chunk
                .iter()
                .map(|item| {
                    let mut s = 0usize;
                    let _ = item;
                    s += 32;
                    s += 32;
                    s
                })
                .sum();
            let mut payload = String::with_capacity(est);
            #[allow(unused_imports)]
            use std::fmt::Write;
            for item in chunk {
                {
                    let v = &item.group_id;
                    write!(&mut payload, "{}", v)
                        .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))?;
                }
                payload.push(',');
                {
                    let v = &item.user_id;
                    write!(&mut payload, "{}", v)
                        .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))?;
                }
                payload.push('\n');
                const MAX_COPY_VALUE_SIZE: usize = 100 * 1024 * 1024;
                if payload.len() > MAX_COPY_VALUE_SIZE {
                    return Err(sqlx::Error::Protocol(
                        format!(
                            "COPY payload exceeds {} bytes limit per chunk",
                            MAX_COPY_VALUE_SIZE
                        )
                        .into(),
                    ));
                }
                if payload.len() > 10 * 1024 * 1024 {
                    {
                        copy_in.send(payload.as_bytes()).await?;
                        payload.clear();
                    }
                }
            }
            if !payload.is_empty() {
                {
                    copy_in.send(payload.as_bytes()).await?;
                }
            }
        }
        copy_in.finish().await?;
        Ok(items.len() as u64)
    }

    pub async fn delete_by_group_id_and_user_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    >(
        executor: E,
        group_id: i32,
        user_id: i32,
    ) -> sqlx::Result<u64> {
        let query = r#"DELETE FROM "user_groups" WHERE "group_id" = $1 AND "user_id" = $2"#;
        let result = sqlx::query::<sqlx::Postgres>(query)
            .bind(group_id)
            .bind(user_id)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }
}
