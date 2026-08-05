#[cfg(not(feature = "validation"))]
compile_error!(
    "Daox: the 'validation' feature is disabled. \
     Format regex checks in validate() will be skipped. \
     Enable with: features = [\"validation\"]"
);
#[allow(clippy::all)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PostgresUserPreferences {
    pub theme: Option<String>,
    pub user_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostgresUserPreferencesOrderBy {
    ThemeAsc,
    ThemeDesc,
    UserIdAsc,
    UserIdDesc,
}

impl PostgresUserPreferencesOrderBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            PostgresUserPreferencesOrderBy::ThemeAsc => r#""theme" ASC"#,
            PostgresUserPreferencesOrderBy::ThemeDesc => r#""theme" DESC"#,
            PostgresUserPreferencesOrderBy::UserIdAsc => r#""user_id" ASC"#,
            PostgresUserPreferencesOrderBy::UserIdDesc => r#""user_id" DESC"#,
        }
    }
}

#[allow(clippy::all)]
impl PostgresUserPreferences {
    #[allow(unused_comparisons, unused_mut)]
    pub fn validate(&self) -> Result<(), Vec<String>> {
        #[cfg(not(feature = "validation"))]
        {
            // Formats validation is disabled
        }
        let mut errors = Vec::new();
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
        let query = r#"SELECT COUNT(*) FROM "user_preferences""#;
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
            .bind("user_preferences")
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
        let query =
            r#"SELECT "theme", "user_id" FROM "user_preferences" ORDER BY "user_id" ASC LIMIT $1"#;
        sqlx::query_as::<_, Self>(query).bind(limit).fetch(executor)
    }

    pub async fn get_by_user_id<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        executor: E,
        user_id: i32,
    ) -> sqlx::Result<Option<Self>> {
        let query = r#"SELECT "theme", "user_id" FROM "user_preferences" WHERE "user_id" = $1"#;
        sqlx::query_as::<_, Self>(query)
            .bind(user_id)
            .fetch_optional(executor)
            .await
    }

    pub async fn exists_by_user_id<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        executor: E,
        user_id: i32,
    ) -> sqlx::Result<bool> {
        let query = r#"SELECT 1 FROM "user_preferences" WHERE "user_id" = $1 LIMIT 1"#;
        let exists: Option<(i32,)> = sqlx::query_as(query)
            .bind(user_id)
            .fetch_optional(executor)
            .await?;
        Ok(exists.is_some())
    }

    pub async fn list_by_cursor<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        executor: E,
        last_id: i32,
        limit: u32,
    ) -> sqlx::Result<Vec<Self>> {
        let limit = limit.clamp(1, 10000);
        let query = r#"SELECT "theme", "user_id" FROM "user_preferences" WHERE "user_id" > $1 ORDER BY "user_id" ASC LIMIT $2"#;
        sqlx::query_as::<_, Self>(query)
            .bind(last_id)
            .bind(limit as i64)
            .fetch_all(executor)
            .await
    }

    pub async fn insert_unchecked<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"INSERT INTO "user_preferences" ("theme", "user_id") VALUES ($1, $2)"#;
        let result = sqlx::query::<sqlx::Postgres>(query)
            .bind(&self.theme)
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
                r#"COPY "user_preferences" ("theme", "user_id") FROM STDIN WITH (FORMAT csv)"#,
            )
            .await?;
        for chunk in items.chunks(1000) {
            let est: usize = chunk
                .iter()
                .map(|item| {
                    let mut s = 0usize;
                    let _ = item;
                    s += item.theme.as_ref().map_or(1, |v| v.len() + 2);
                    s += 32;
                    s
                })
                .sum();
            let mut payload = String::with_capacity(est);
            #[allow(unused_imports)]
            use std::fmt::Write;
            for item in chunk {
                if let Some(v) = &item.theme {
                    payload.push('"');
                    for c in v.chars() {
                        if c == '"' {
                            payload.push_str("\"\"");
                        } else {
                            payload.push(c);
                        }
                    }
                    payload.push('"');
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

    pub async fn upsert_unchecked<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"INSERT INTO "user_preferences" ("theme", "user_id") VALUES ($1, $2) ON CONFLICT ("user_id") DO UPDATE SET "theme" = EXCLUDED."theme""#;
        let result = sqlx::query::<sqlx::Postgres>(query)
            .bind(&self.theme)
            .bind(&self.user_id)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn upsert<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        if let Err(e) = self.validate() {
            return Err(sqlx::Error::Protocol(e.join(", ").into()));
        }
        self.upsert_unchecked(executor).await
    }

    /// Upserts a batch of records.
    /// WARNING: To guarantee atomicity across all chunks, you MUST pass an explicit `sqlx::Transaction` as the `executor`.
    pub async fn upsert_batch<'e>(
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
                        "upsert_batch: item {} failed validation: {}",
                        idx,
                        e.join(", ")
                    )
                    .into(),
                ));
            }
        }
        let chunk_size = 65535 / 2;
        let mut total_affected = 0;
        for chunk in items.chunks(chunk_size.max(1)) {
            let mut qb: sqlx::QueryBuilder<sqlx::Postgres> =
                sqlx::QueryBuilder::new(r#"INSERT INTO "user_preferences" ("theme", "user_id") "#);
            qb.push_values(chunk, |mut b, item| {
                b.push_bind(&item.theme);
                b.push_bind(&item.user_id);
            });
            qb.push(r#" ON CONFLICT ("user_id") DO UPDATE SET "theme" = EXCLUDED."theme""#);
            let result = qb.build().execute(&mut **executor).await?;
            total_affected += result.rows_affected();
        }
        Ok(total_affected)
    }

    pub async fn update_unchecked_by_user_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    >(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query_str = r#"UPDATE "user_preferences" SET "theme" = $1 WHERE "user_id" = $2"#;
        let mut query = sqlx::query::<sqlx::Postgres>(query_str);
        query = query.bind(&self.theme);
        query = query.bind(&self.user_id);
        let result = query.execute(executor).await?;
        Ok(result.rows_affected())
    }

    pub async fn update_by_user_id<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        if let Err(e) = self.validate() {
            return Err(sqlx::Error::Protocol(e.join(", ").into()));
        }
        self.update_unchecked_by_user_id(executor).await
    }

    pub async fn delete_by_user_id<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        executor: E,
        user_id: i32,
    ) -> sqlx::Result<u64> {
        let query = r#"DELETE FROM "user_preferences" WHERE "user_id" = $1"#;
        let result = sqlx::query::<sqlx::Postgres>(query)
            .bind(user_id)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_many_by_user_id<'e>(
        executor: &mut sqlx::Transaction<'e, sqlx::Postgres>,
        ids: &[i32],
    ) -> sqlx::Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }
        let mut total_affected = 0;
        let chunk_size = 5000_usize.min(65535);
        for chunk in ids.chunks(chunk_size) {
            let mut qb: sqlx::QueryBuilder<sqlx::Postgres> =
                sqlx::QueryBuilder::new(r#"DELETE FROM "user_preferences" WHERE "user_id" IN "#);
            qb.push("(");
            let mut sep = qb.separated(", ");
            for id in chunk {
                sep.push_bind(id);
            }
            sep.push_unseparated(")");
            let result = qb.build().execute(&mut **executor).await?;
            total_affected += result.rows_affected();
        }
        Ok(total_affected)
    }

    #[allow(unused_assignments, unused_comparisons, unused_mut, unused_variables)]
    pub async fn update_partial_by_user_id<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
        executor: E,
        user_id: i32,
        patch: &PostgresUserPreferencesPatch,
    ) -> sqlx::Result<u64> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(val) = &patch.theme {
            if let Some(v) = val.as_ref() {}
        }
        if !errors.is_empty() {
            return Err(sqlx::Error::Protocol(errors.join(", ").into()));
        }
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> =
            sqlx::QueryBuilder::new("UPDATE \"user_preferences\" SET ");
        let mut first = true;
        if let Some(val) = &patch.theme {
            if !first {
                qb.push(", ");
            }
            qb.push("\"theme\" = ");
            qb.push_bind(val.clone());
            first = false;
        }
        if first {
            return Ok(0);
        }
        qb.push(" WHERE \"user_id\" = ");
        qb.push_bind(user_id);
        let result = qb.build().execute(executor).await?;
        Ok(result.rows_affected())
    }
}

#[allow(clippy::all)]
#[derive(Debug, Clone, Default)]
pub struct PostgresUserPreferencesPatch {
    pub theme: Option<Option<String>>,
}
