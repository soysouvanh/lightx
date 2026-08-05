#[cfg(not(feature = "validation"))]
compile_error!(
    "Daox: the 'validation' feature is disabled. \
     Format regex checks in validate() will be skipped. \
     Enable with: features = [\"validation\"]"
);
#[allow(clippy::all)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MysqlUserGroups {
    pub group_id: i32,
    pub user_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MysqlUserGroupsOrderBy {
    GroupIdAsc,
    GroupIdDesc,
    UserIdAsc,
    UserIdDesc,
}

impl MysqlUserGroupsOrderBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            MysqlUserGroupsOrderBy::GroupIdAsc => r#"`group_id` ASC"#,
            MysqlUserGroupsOrderBy::GroupIdDesc => r#"`group_id` DESC"#,
            MysqlUserGroupsOrderBy::UserIdAsc => r#"`user_id` ASC"#,
            MysqlUserGroupsOrderBy::UserIdDesc => r#"`user_id` DESC"#,
        }
    }
}

#[allow(clippy::all)]
impl MysqlUserGroups {
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
    pub async fn count<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"SELECT COUNT(*) FROM `user_groups`"#;
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
            .bind("user_groups")
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
        let query = r#"SELECT `group_id`, `user_id` FROM `user_groups` ORDER BY `group_id` ASC, `user_id` ASC LIMIT ?"#;
        sqlx::query_as::<_, Self>(query).bind(limit).fetch(executor)
    }

    pub async fn get_by_group_id_and_user_id<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        executor: E,
        group_id: i32,
        user_id: i32,
    ) -> sqlx::Result<Option<Self>> {
        let query = r#"SELECT `group_id`, `user_id` FROM `user_groups` WHERE `group_id` = ? AND `user_id` = ?"#;
        sqlx::query_as::<_, Self>(query)
            .bind(group_id)
            .bind(user_id)
            .fetch_optional(executor)
            .await
    }

    pub async fn exists_by_group_id_and_user_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::MySql>,
    >(
        executor: E,
        group_id: i32,
        user_id: i32,
    ) -> sqlx::Result<bool> {
        let query = r#"SELECT 1 FROM `user_groups` WHERE `group_id` = ? AND `user_id` = ? LIMIT 1"#;
        let exists: Option<(i32,)> = sqlx::query_as(query)
            .bind(group_id)
            .bind(user_id)
            .fetch_optional(executor)
            .await?;
        Ok(exists.is_some())
    }

    pub async fn insert_unchecked<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query = r#"INSERT INTO `user_groups` (`group_id`, `user_id`) VALUES (?, ?)"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(&self.group_id)
            .bind(&self.user_id)
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
        let chunk_size = 65535 / 2;
        let mut total_affected = 0;
        for chunk in items.chunks(chunk_size.max(1)) {
            let mut qb: sqlx::QueryBuilder<sqlx::MySql> =
                sqlx::QueryBuilder::new(r#"INSERT INTO `user_groups` (`group_id`, `user_id`) "#);
            qb.push_values(chunk, |mut b, item| {
                b.push_bind(&item.group_id);
                b.push_bind(&item.user_id);
            });
            let result = qb.build().execute(&mut **executor).await?;
            total_affected += result.rows_affected();
        }
        Ok(total_affected)
    }

    pub async fn delete_by_group_id_and_user_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::MySql>,
    >(
        executor: E,
        group_id: i32,
        user_id: i32,
    ) -> sqlx::Result<u64> {
        let query = r#"DELETE FROM `user_groups` WHERE `group_id` = ? AND `user_id` = ?"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(group_id)
            .bind(user_id)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }
}
