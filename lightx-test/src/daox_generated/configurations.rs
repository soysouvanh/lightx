#[cfg(not(feature = "validation"))]
compile_error!(
    "Daox: the 'validation' feature is disabled. \
     Format regex checks in validate() will be skipped. \
     Enable with: features = [\"validation\"]"
);
#[allow(clippy::all)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Configurations {
    pub id: i32,
    pub r#match: Option<String>,
    pub r#type: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationsOrderBy {
    IdAsc,
    IdDesc,
    MatchAsc,
    MatchDesc,
    TypeAsc,
    TypeDesc,
    ValueAsc,
    ValueDesc,
}

impl ConfigurationsOrderBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigurationsOrderBy::IdAsc => r#"`id` ASC"#,
            ConfigurationsOrderBy::IdDesc => r#"`id` DESC"#,
            ConfigurationsOrderBy::MatchAsc => r#"`match` ASC"#,
            ConfigurationsOrderBy::MatchDesc => r#"`match` DESC"#,
            ConfigurationsOrderBy::TypeAsc => r#"`type` ASC"#,
            ConfigurationsOrderBy::TypeDesc => r#"`type` DESC"#,
            ConfigurationsOrderBy::ValueAsc => r#"`value` ASC"#,
            ConfigurationsOrderBy::ValueDesc => r#"`value` DESC"#,
        }
    }
}

#[allow(clippy::all)]
impl Configurations {
    #[allow(unused_comparisons, unused_mut)]
    pub fn validate(&self) -> Result<(), Vec<String>> {
        #[cfg(not(feature = "validation"))]
        {
            // Formats validation is disabled
        }
        let mut errors = Vec::new();
        if let Some(v) = Some(&self.id) {
            if (*v as i128) < (0 as i128) {
                errors.push("id: minimum value '0' not met".into());
            }
        }
        if let Some(v) = Some(&self.id) {
            if (*v as i128) > (2147483647 as i128) {
                errors.push("id: maximum value '2147483647' exceeded".into());
            }
        }
        if let Some(v) = self.r#match.as_ref() {
            if v.len() > 255 {
                errors.push("r#match: exceeds max_length 255".into());
            }
        }
        #[cfg(feature = "validation")]
        if let Some(v) = self.r#match.as_ref() {
            static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
            let re = RE.get_or_init(|| regex::Regex::new("^[À-ÿA-Za-z0-9_ -]*$").ok());
            match re {
                Some(re) => {
                    if !re.is_match(v) {
                        errors.push("r#match: format constraint not met".into());
                    }
                }
                None => {
                    errors.push("r#match: configured regex is invalid".into());
                }
            }
        }
        if let Some(v) = Some(&self.r#type) {
            if v.len() < 1 {
                errors.push("r#type: min_length 1 not met".into());
            }
        }
        if let Some(v) = Some(&self.r#type) {
            if v.len() > 50 {
                errors.push("r#type: exceeds max_length 50".into());
            }
        }
        #[cfg(feature = "validation")]
        if let Some(v) = Some(&self.r#type) {
            static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
            let re = RE.get_or_init(|| regex::Regex::new("^[À-ÿA-Za-z0-9_ -]*$").ok());
            match re {
                Some(re) => {
                    if !re.is_match(v) {
                        errors.push("r#type: format constraint not met".into());
                    }
                }
                None => {
                    errors.push("r#type: configured regex is invalid".into());
                }
            }
        }
        if let Some(v) = self.value.as_ref() {
            if v.len() > 65535 {
                errors.push("value: exceeds max_length 65535".into());
            }
        }
        #[cfg(feature = "validation")]
        if let Some(v) = self.value.as_ref() {
            static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
            let re = RE.get_or_init(|| regex::Regex::new("^[À-ÿA-Za-z0-9_ -]*$").ok());
            match re {
                Some(re) => {
                    if !re.is_match(v) {
                        errors.push("value: format constraint not met".into());
                    }
                }
                None => {
                    errors.push("value: configured regex is invalid".into());
                }
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
        let query = r#"SELECT COUNT(*) FROM `configurations`"#;
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
            .bind("configurations")
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
        let query = r#"SELECT `id`, `match`, `type`, `value` FROM `configurations` ORDER BY `id` ASC LIMIT ?"#;
        sqlx::query_as::<_, Self>(query).bind(limit).fetch(executor)
    }

    pub async fn get_by_id<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        id: i32,
    ) -> sqlx::Result<Option<Self>> {
        let query = r#"SELECT `id`, `match`, `type`, `value` FROM `configurations` WHERE `id` = ?"#;
        sqlx::query_as::<_, Self>(query)
            .bind(id)
            .fetch_optional(executor)
            .await
    }

    pub async fn exists_by_id<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        id: i32,
    ) -> sqlx::Result<bool> {
        let query = r#"SELECT 1 FROM `configurations` WHERE `id` = ? LIMIT 1"#;
        let exists: Option<(i32,)> = sqlx::query_as(query)
            .bind(id)
            .fetch_optional(executor)
            .await?;
        Ok(exists.is_some())
    }

    pub async fn list_by_cursor<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        last_id: i32,
        limit: u32,
    ) -> sqlx::Result<Vec<Self>> {
        let limit = limit.clamp(1, 10000);
        let query = r#"SELECT `id`, `match`, `type`, `value` FROM `configurations` WHERE `id` > ? ORDER BY `id` ASC LIMIT ?"#;
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
        let query = r#"INSERT INTO `configurations` (`match`, `type`, `value`) VALUES (?, ?, ?)"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(&self.r#match)
            .bind(&self.r#type)
            .bind(&self.value)
            .execute(executor)
            .await?;
        Ok(result.last_insert_id())
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
        let chunk_size = 65535 / 3;
        let mut total_affected = 0;
        for chunk in items.chunks(chunk_size.max(1)) {
            let mut qb: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
                r#"INSERT INTO `configurations` (`match`, `type`, `value`) "#,
            );
            qb.push_values(chunk, |mut b, item| {
                b.push_bind(&item.r#match);
                b.push_bind(&item.r#type);
                b.push_bind(&item.value);
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
        let query = r#"INSERT INTO `configurations` (`match`, `type`, `value`) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE `match` = VALUES(`match`), `type` = VALUES(`type`), `value` = VALUES(`value`)"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(&self.r#match)
            .bind(&self.r#type)
            .bind(&self.value)
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
        let chunk_size = 65535 / 3;
        let mut total_affected = 0;
        for chunk in items.chunks(chunk_size.max(1)) {
            let mut qb: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
                r#"INSERT INTO `configurations` (`match`, `type`, `value`) "#,
            );
            qb.push_values(chunk, |mut b, item| {
                b.push_bind(&item.r#match);
                b.push_bind(&item.r#type);
                b.push_bind(&item.value);
            });
            qb.push(r#" ON DUPLICATE KEY UPDATE `match` = VALUES(`match`), `type` = VALUES(`type`), `value` = VALUES(`value`)"#);
            let result = qb.build().execute(&mut **executor).await?;
            total_affected += result.rows_affected();
        }
        Ok(total_affected)
    }

    pub async fn update_unchecked_by_id<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query_str =
            r#"UPDATE `configurations` SET `match` = ?, `type` = ?, `value` = ? WHERE `id` = ?"#;
        let mut query = sqlx::query::<sqlx::MySql>(query_str);
        query = query.bind(&self.r#match);
        query = query.bind(&self.r#type);
        query = query.bind(&self.value);
        query = query.bind(&self.id);
        let result = query.execute(executor).await?;
        Ok(result.rows_affected())
    }

    pub async fn update_by_id<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        if let Err(e) = self.validate() {
            return Err(sqlx::Error::Protocol(e.join(", ").into()));
        }
        self.update_unchecked_by_id(executor).await
    }

    pub async fn delete_by_id<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        id: i32,
    ) -> sqlx::Result<u64> {
        let query = r#"DELETE FROM `configurations` WHERE `id` = ?"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(id)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_many_by_id<'e>(
        executor: &mut sqlx::Transaction<'e, sqlx::MySql>,
        ids: &[i32],
    ) -> sqlx::Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }
        let mut total_affected = 0;
        let chunk_size = 5000_usize.min(65535);
        for chunk in ids.chunks(chunk_size) {
            let mut qb: sqlx::QueryBuilder<sqlx::MySql> =
                sqlx::QueryBuilder::new(r#"DELETE FROM `configurations` WHERE `id` IN "#);
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
    pub async fn update_partial_by_id<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        id: i32,
        patch: &ConfigurationsPatch,
    ) -> sqlx::Result<u64> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(val) = &patch.r#match {
            if let Some(v) = val.as_ref() {
                if v.len() > 255 {
                    errors.push("r#match: exceeds max_length 255".into());
                }
                #[cfg(feature = "validation")]
                {
                    static RE: std::sync::OnceLock<Option<regex::Regex>> =
                        std::sync::OnceLock::new();
                    let re = RE.get_or_init(|| regex::Regex::new("^[À-ÿA-Za-z0-9_ -]*$").ok());
                    if let Some(re) = re {
                        if !re.is_match(v) {
                            errors.push("r#match: format constraint not met".into());
                        }
                    }
                }
            }
        }
        if let Some(val) = &patch.r#type {
            if val.len() < 1 {
                errors.push("r#type: min_length 1 not met".into());
            }
            if val.len() > 50 {
                errors.push("r#type: exceeds max_length 50".into());
            }
            #[cfg(feature = "validation")]
            {
                static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
                let re = RE.get_or_init(|| regex::Regex::new("^[À-ÿA-Za-z0-9_ -]*$").ok());
                if let Some(re) = re {
                    if !re.is_match(val) {
                        errors.push("r#type: format constraint not met".into());
                    }
                }
            }
        }
        if let Some(val) = &patch.value {
            if let Some(v) = val.as_ref() {
                if v.len() > 65535 {
                    errors.push("value: exceeds max_length 65535".into());
                }
                #[cfg(feature = "validation")]
                {
                    static RE: std::sync::OnceLock<Option<regex::Regex>> =
                        std::sync::OnceLock::new();
                    let re = RE.get_or_init(|| regex::Regex::new("^[À-ÿA-Za-z0-9_ -]*$").ok());
                    if let Some(re) = re {
                        if !re.is_match(v) {
                            errors.push("value: format constraint not met".into());
                        }
                    }
                }
            }
        }
        if !errors.is_empty() {
            return Err(sqlx::Error::Protocol(errors.join(", ").into()));
        }
        let mut qb: sqlx::QueryBuilder<sqlx::MySql> =
            sqlx::QueryBuilder::new("UPDATE `configurations` SET ");
        let mut first = true;
        if let Some(val) = &patch.r#match {
            if !first {
                qb.push(", ");
            }
            qb.push("`match` = ");
            qb.push_bind(val.clone());
            first = false;
        }
        if let Some(val) = &patch.r#type {
            if !first {
                qb.push(", ");
            }
            qb.push("`type` = ");
            qb.push_bind(val.clone());
            first = false;
        }
        if let Some(val) = &patch.value {
            if !first {
                qb.push(", ");
            }
            qb.push("`value` = ");
            qb.push_bind(val.clone());
            first = false;
        }
        if first {
            return Ok(0);
        }
        qb.push(" WHERE `id` = ");
        qb.push_bind(id);
        let result = qb.build().execute(executor).await?;
        Ok(result.rows_affected())
    }
}

#[allow(clippy::all)]
#[derive(Debug, Clone, Default)]
pub struct ConfigurationsPatch {
    pub r#match: Option<Option<String>>,
    pub r#type: Option<String>,
    pub value: Option<Option<String>>,
}
