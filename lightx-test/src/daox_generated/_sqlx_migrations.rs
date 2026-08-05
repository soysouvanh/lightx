#[cfg(not(feature = "validation"))]
compile_error!(
    "Daox: the 'validation' feature is disabled. \
     Format regex checks in validate() will be skipped. \
     Enable with: features = [\"validation\"]"
);
#[allow(clippy::all)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxMigrations {
    pub checksum: Vec<u8>,
    pub description: String,
    pub execution_time: i64,
    pub installed_on: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub version: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlxMigrationsOrderBy {
    ChecksumAsc,
    ChecksumDesc,
    DescriptionAsc,
    DescriptionDesc,
    ExecutionTimeAsc,
    ExecutionTimeDesc,
    InstalledOnAsc,
    InstalledOnDesc,
    SuccessAsc,
    SuccessDesc,
    VersionAsc,
    VersionDesc,
}

impl SqlxMigrationsOrderBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            SqlxMigrationsOrderBy::ChecksumAsc => r#"`checksum` ASC"#,
            SqlxMigrationsOrderBy::ChecksumDesc => r#"`checksum` DESC"#,
            SqlxMigrationsOrderBy::DescriptionAsc => r#"`description` ASC"#,
            SqlxMigrationsOrderBy::DescriptionDesc => r#"`description` DESC"#,
            SqlxMigrationsOrderBy::ExecutionTimeAsc => r#"`execution_time` ASC"#,
            SqlxMigrationsOrderBy::ExecutionTimeDesc => r#"`execution_time` DESC"#,
            SqlxMigrationsOrderBy::InstalledOnAsc => r#"`installed_on` ASC"#,
            SqlxMigrationsOrderBy::InstalledOnDesc => r#"`installed_on` DESC"#,
            SqlxMigrationsOrderBy::SuccessAsc => r#"`success` ASC"#,
            SqlxMigrationsOrderBy::SuccessDesc => r#"`success` DESC"#,
            SqlxMigrationsOrderBy::VersionAsc => r#"`version` ASC"#,
            SqlxMigrationsOrderBy::VersionDesc => r#"`version` DESC"#,
        }
    }
}

#[allow(clippy::all)]
impl SqlxMigrations {
    #[allow(unused_comparisons, unused_mut)]
    pub fn validate(&self) -> Result<(), Vec<String>> {
        #[cfg(not(feature = "validation"))]
        {
            // Formats validation is disabled
        }
        let mut errors = Vec::new();
        if let Some(v) = Some(&self.checksum) {
            if v.len() < 1 {
                errors.push("checksum: min_length 1 not met".into());
            }
        }
        if let Some(v) = Some(&self.checksum) {
            if v.len() > 65535 {
                errors.push("checksum: exceeds max_length 65535".into());
            }
        }
        if let Some(v) = Some(&self.description) {
            if v.len() < 1 {
                errors.push("description: min_length 1 not met".into());
            }
        }
        if let Some(v) = Some(&self.description) {
            if v.len() > 65535 {
                errors.push("description: exceeds max_length 65535".into());
            }
        }
        #[cfg(feature = "validation")]
        if let Some(v) = Some(&self.description) {
            static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
            let re = RE.get_or_init(|| regex::Regex::new("^[À-ÿA-Za-z0-9_ -]*$").ok());
            match re {
                Some(re) => {
                    if !re.is_match(v) {
                        errors.push("description: format constraint not met".into());
                    }
                }
                None => {
                    errors.push("description: configured regex is invalid".into());
                }
            }
        }
        if let Some(v) = Some(&self.execution_time) {
            if (*v as i128) < (0 as i128) {
                errors.push("execution_time: minimum value '0' not met".into());
            }
        }
        if let Some(v) = Some(&self.execution_time) {
            if (*v as i128) > (9223372036854775807 as i128) {
                errors.push("execution_time: maximum value '9223372036854775807' exceeded".into());
            }
        }
        if let Some(v) = Some(&self.version) {
            if (*v as i128) < (0 as i128) {
                errors.push("version: minimum value '0' not met".into());
            }
        }
        if let Some(v) = Some(&self.version) {
            if (*v as i128) > (9223372036854775807 as i128) {
                errors.push("version: maximum value '9223372036854775807' exceeded".into());
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
    pub async fn count<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"SELECT COUNT(*) FROM `_sqlx_migrations`"#;
        let (count,): (i64,) = sqlx::query_as(query).fetch_one(executor).await?;
        Ok(count as u64)
    }

    /// Returns an approximate total number of rows in the table using database statistics (O(1)).
    /// WARNING (MySQL): For InnoDB tables, this value is an estimate and can vary significantly from the actual count.
    pub async fn approximate_count<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"SELECT table_rows FROM information_schema.tables WHERE table_name = ? AND table_schema = DATABASE()"#;
        let count: Option<(u64,)> = sqlx::query_as(query)
            .bind("_sqlx_migrations")
            .fetch_optional(executor)
            .await?;
        Ok(count.map(|(c,)| c).unwrap_or(0))
    }

    /// Streams rows from the table, ordered by the primary key.
    /// **⚠️ Performance Warning:** Streaming a whole table without a limit or timeout can cause connection pool starvation.
    /// A `limit` parameter is now mandatory to prevent Unbounded Streaming DoS. Timeouts are managed by the underlying sqlx `AnyPoolOptions` settings.
    #[deprecated(
        since = "0.2.0",
        note = "Use cursor-based pagination instead to prevent pool starvation."
    )]
    pub fn stream_all<'e, E: sqlx::Executor<'e, Database = sqlx::MySql> + 'e>(
        executor: E,
        limit: i64,
    ) -> impl futures::Stream<Item = sqlx::Result<Self>> + 'e {
        let limit = limit.clamp(1, 10000);
        let query = r#"SELECT `checksum`, `description`, `execution_time`, `installed_on`, `success`, `version` FROM `_sqlx_migrations` ORDER BY `version` ASC LIMIT ?"#;
        sqlx::query_as::<_, Self>(query).bind(limit).fetch(executor)
    }

    pub async fn get_by_version<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        version: i64,
    ) -> sqlx::Result<Option<Self>> {
        let query = r#"SELECT `checksum`, `description`, `execution_time`, `installed_on`, `success`, `version` FROM `_sqlx_migrations` WHERE `version` = ?"#;
        sqlx::query_as::<_, Self>(query)
            .bind(version)
            .fetch_optional(executor)
            .await
    }

    pub async fn exists_by_version<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        version: i64,
    ) -> sqlx::Result<bool> {
        let query = r#"SELECT 1 FROM `_sqlx_migrations` WHERE `version` = ? LIMIT 1"#;
        let exists: Option<(i32,)> = sqlx::query_as(query)
            .bind(version)
            .fetch_optional(executor)
            .await?;
        Ok(exists.is_some())
    }

    pub async fn list_by_cursor<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        last_id: i64,
        limit: u32,
    ) -> sqlx::Result<Vec<Self>> {
        let limit = limit.clamp(1, 10000);
        let query = r#"SELECT `checksum`, `description`, `execution_time`, `installed_on`, `success`, `version` FROM `_sqlx_migrations` WHERE `version` > ? ORDER BY `version` ASC LIMIT ?"#;
        sqlx::query_as::<_, Self>(query)
            .bind(last_id)
            .bind(limit as i64)
            .fetch_all(executor)
            .await
    }

    pub async fn insert_unchecked<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"INSERT INTO `_sqlx_migrations` (`checksum`, `description`, `execution_time`, `success`, `version`) VALUES (?, ?, ?, ?, ?)"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(&self.checksum)
            .bind(&self.description)
            .bind(&self.execution_time)
            .bind(&self.success)
            .bind(&self.version)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn insert<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        if let Err(e) = self.validate() {
            return Err(sqlx::Error::Protocol(e.join(", ").into()));
        }
        self.insert_unchecked(executor).await
    }

    /// Inserts a batch of records.
    /// WARNING: To guarantee atomicity across all chunks, you MUST pass an explicit `sqlx::Transaction` as the `executor`.
    pub async fn insert_batch<'e>(
        executor: &mut sqlx::Transaction<'e, sqlx::MySql>,
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
        let chunk_size = 65535 / 5;
        let mut total_affected = 0;
        for chunk in items.chunks(chunk_size.max(1)) {
            let mut qb: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
                r#"INSERT INTO `_sqlx_migrations` (`checksum`, `description`, `execution_time`, `success`, `version`) "#,
            );
            qb.push_values(chunk, |mut b, item| {
                b.push_bind(&item.checksum);
                b.push_bind(&item.description);
                b.push_bind(&item.execution_time);
                b.push_bind(&item.success);
                b.push_bind(&item.version);
            });
            let result = qb.build().execute(&mut **executor).await?;
            total_affected += result.rows_affected();
        }
        Ok(total_affected)
    }

    pub async fn upsert_unchecked<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"INSERT INTO `_sqlx_migrations` (`checksum`, `description`, `execution_time`, `success`, `version`) VALUES (?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE `checksum` = VALUES(`checksum`), `description` = VALUES(`description`), `execution_time` = VALUES(`execution_time`), `success` = VALUES(`success`)"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(&self.checksum)
            .bind(&self.description)
            .bind(&self.execution_time)
            .bind(&self.success)
            .bind(&self.version)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn upsert<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
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
        executor: &mut sqlx::Transaction<'e, sqlx::MySql>,
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
        let chunk_size = 65535 / 5;
        let mut total_affected = 0;
        for chunk in items.chunks(chunk_size.max(1)) {
            let mut qb: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
                r#"INSERT INTO `_sqlx_migrations` (`checksum`, `description`, `execution_time`, `success`, `version`) "#,
            );
            qb.push_values(chunk, |mut b, item| {
                b.push_bind(&item.checksum);
                b.push_bind(&item.description);
                b.push_bind(&item.execution_time);
                b.push_bind(&item.success);
                b.push_bind(&item.version);
            });
            qb.push(r#" ON DUPLICATE KEY UPDATE `checksum` = VALUES(`checksum`), `description` = VALUES(`description`), `execution_time` = VALUES(`execution_time`), `success` = VALUES(`success`)"#);
            let result = qb.build().execute(&mut **executor).await?;
            total_affected += result.rows_affected();
        }
        Ok(total_affected)
    }

    pub async fn update_unchecked_by_version<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query_str = r#"UPDATE `_sqlx_migrations` SET `checksum` = ?, `description` = ?, `execution_time` = ?, `success` = ? WHERE `version` = ?"#;
        let mut query = sqlx::query::<sqlx::MySql>(query_str);
        query = query.bind(&self.checksum);
        query = query.bind(&self.description);
        query = query.bind(&self.execution_time);
        query = query.bind(&self.success);
        query = query.bind(&self.version);
        let result = query.execute(executor).await?;
        Ok(result.rows_affected())
    }

    pub async fn update_by_version<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        if let Err(e) = self.validate() {
            return Err(sqlx::Error::Protocol(e.join(", ").into()));
        }
        self.update_unchecked_by_version(executor).await
    }

    pub async fn delete_by_version<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        version: i64,
    ) -> sqlx::Result<u64> {
        let query = r#"DELETE FROM `_sqlx_migrations` WHERE `version` = ?"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(version)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_many_by_version<'e>(
        executor: &mut sqlx::Transaction<'e, sqlx::MySql>,
        ids: &[i64],
    ) -> sqlx::Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }
        let mut total_affected = 0;
        let chunk_size = 5000_usize.min(65535);
        for chunk in ids.chunks(chunk_size) {
            let mut qb: sqlx::QueryBuilder<sqlx::MySql> =
                sqlx::QueryBuilder::new(r#"DELETE FROM `_sqlx_migrations` WHERE `version` IN "#);
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
    pub async fn update_partial_by_version<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        version: i64,
        patch: &SqlxMigrationsPatch,
    ) -> sqlx::Result<u64> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(val) = &patch.checksum {
            if val.len() < 1 {
                errors.push("checksum: min_length 1 not met".into());
            }
            if val.len() > 65535 {
                errors.push("checksum: exceeds max_length 65535".into());
            }
        }
        if let Some(val) = &patch.description {
            if val.len() < 1 {
                errors.push("description: min_length 1 not met".into());
            }
            if val.len() > 65535 {
                errors.push("description: exceeds max_length 65535".into());
            }
            #[cfg(feature = "validation")]
            {
                static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
                let re = RE.get_or_init(|| regex::Regex::new("^[À-ÿA-Za-z0-9_ -]*$").ok());
                if let Some(re) = re {
                    if !re.is_match(val) {
                        errors.push("description: format constraint not met".into());
                    }
                }
            }
        }
        if let Some(val) = &patch.execution_time {
            if (*val as i128) < (0 as i128) {
                errors.push("execution_time: minimum value '0' not met".into());
            }
            if (*val as i128) > (9223372036854775807 as i128) {
                errors.push("execution_time: exceeds max_value '9223372036854775807'".into());
            }
        }
        if let Some(val) = &patch.success {}
        if !errors.is_empty() {
            return Err(sqlx::Error::Protocol(errors.join(", ").into()));
        }
        let mut qb: sqlx::QueryBuilder<sqlx::MySql> =
            sqlx::QueryBuilder::new("UPDATE `_sqlx_migrations` SET ");
        let mut first = true;
        if let Some(val) = &patch.checksum {
            if !first {
                qb.push(", ");
            }
            qb.push("`checksum` = ");
            qb.push_bind(val.clone());
            first = false;
        }
        if let Some(val) = &patch.description {
            if !first {
                qb.push(", ");
            }
            qb.push("`description` = ");
            qb.push_bind(val.clone());
            first = false;
        }
        if let Some(val) = &patch.execution_time {
            if !first {
                qb.push(", ");
            }
            qb.push("`execution_time` = ");
            qb.push_bind(val.clone());
            first = false;
        }
        if let Some(val) = &patch.success {
            if !first {
                qb.push(", ");
            }
            qb.push("`success` = ");
            qb.push_bind(val.clone());
            first = false;
        }
        if first {
            return Ok(0);
        }
        qb.push(" WHERE `version` = ");
        qb.push_bind(version);
        let result = qb.build().execute(executor).await?;
        Ok(result.rows_affected())
    }
}

#[allow(clippy::all)]
#[derive(Debug, Clone, Default)]
pub struct SqlxMigrationsPatch {
    pub checksum: Option<Vec<u8>>,
    pub description: Option<String>,
    pub execution_time: Option<i64>,
    pub success: Option<bool>,
}
