#[cfg(not(feature = "validation"))]
compile_error!(
    "Daox: the 'validation' feature is disabled. \
     Format regex checks in validate() will be skipped. \
     Enable with: features = [\"validation\"]"
);
#[allow(clippy::all)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OrderItems {
    pub order_id: i64,
    pub product_id: i64,
    pub quantity: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderItemsOrderBy {
    OrderIdAsc,
    OrderIdDesc,
    ProductIdAsc,
    ProductIdDesc,
    QuantityAsc,
    QuantityDesc,
}

impl OrderItemsOrderBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderItemsOrderBy::OrderIdAsc => r#"`order_id` ASC"#,
            OrderItemsOrderBy::OrderIdDesc => r#"`order_id` DESC"#,
            OrderItemsOrderBy::ProductIdAsc => r#"`product_id` ASC"#,
            OrderItemsOrderBy::ProductIdDesc => r#"`product_id` DESC"#,
            OrderItemsOrderBy::QuantityAsc => r#"`quantity` ASC"#,
            OrderItemsOrderBy::QuantityDesc => r#"`quantity` DESC"#,
        }
    }
}

#[allow(clippy::all)]
impl OrderItems {
    #[allow(unused_comparisons, unused_mut)]
    pub fn validate(&self) -> Result<(), Vec<String>> {
        #[cfg(not(feature = "validation"))]
        {
            // Formats validation is disabled
        }
        let mut errors = Vec::new();
        if let Some(v) = Some(&self.order_id) {
            if (*v as i128) < (0 as i128) {
                errors.push("order_id: minimum value '0' not met".into());
            }
        }
        if let Some(v) = Some(&self.order_id) {
            if (*v as i128) > (9223372036854775807 as i128) {
                errors.push("order_id: maximum value '9223372036854775807' exceeded".into());
            }
        }
        if let Some(v) = Some(&self.product_id) {
            if (*v as i128) < (0 as i128) {
                errors.push("product_id: minimum value '0' not met".into());
            }
        }
        if let Some(v) = Some(&self.product_id) {
            if (*v as i128) > (9223372036854775807 as i128) {
                errors.push("product_id: maximum value '9223372036854775807' exceeded".into());
            }
        }
        if let Some(v) = Some(&self.quantity) {
            if (*v as i128) < (0 as i128) {
                errors.push("quantity: minimum value '0' not met".into());
            }
        }
        if let Some(v) = Some(&self.quantity) {
            if (*v as i128) > (2147483647 as i128) {
                errors.push("quantity: maximum value '2147483647' exceeded".into());
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
        let query = r#"SELECT COUNT(*) FROM `order_items`"#;
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
            .bind("order_items")
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
        let query = r#"SELECT `order_id`, `product_id`, `quantity` FROM `order_items` ORDER BY `order_id` ASC, `product_id` ASC LIMIT ?"#;
        sqlx::query_as::<_, Self>(query).bind(limit).fetch(executor)
    }

    pub async fn get_by_order_id_and_product_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::MySql>,
    >(
        executor: E,
        order_id: i64,
        product_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        let query = r#"SELECT `order_id`, `product_id`, `quantity` FROM `order_items` WHERE `order_id` = ? AND `product_id` = ?"#;
        sqlx::query_as::<_, Self>(query)
            .bind(order_id)
            .bind(product_id)
            .fetch_optional(executor)
            .await
    }

    pub async fn exists_by_order_id_and_product_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::MySql>,
    >(
        executor: E,
        order_id: i64,
        product_id: i64,
    ) -> sqlx::Result<bool> {
        let query =
            r#"SELECT 1 FROM `order_items` WHERE `order_id` = ? AND `product_id` = ? LIMIT 1"#;
        let exists: Option<(i32,)> = sqlx::query_as(query)
            .bind(order_id)
            .bind(product_id)
            .fetch_optional(executor)
            .await?;
        Ok(exists.is_some())
    }

    pub async fn insert_unchecked<'e, E: sqlx::Executor<'e, Database = sqlx::MySql>>(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query =
            r#"INSERT INTO `order_items` (`order_id`, `product_id`, `quantity`) VALUES (?, ?, ?)"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(&self.order_id)
            .bind(&self.product_id)
            .bind(&self.quantity)
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
        let chunk_size = 65535 / 3;
        let mut total_affected = 0;
        for chunk in items.chunks(chunk_size.max(1)) {
            let mut qb: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
                r#"INSERT INTO `order_items` (`order_id`, `product_id`, `quantity`) "#,
            );
            qb.push_values(chunk, |mut b, item| {
                b.push_bind(&item.order_id);
                b.push_bind(&item.product_id);
                b.push_bind(&item.quantity);
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
        let query = r#"INSERT INTO `order_items` (`order_id`, `product_id`, `quantity`) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE `quantity` = VALUES(`quantity`)"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(&self.order_id)
            .bind(&self.product_id)
            .bind(&self.quantity)
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
                r#"INSERT INTO `order_items` (`order_id`, `product_id`, `quantity`) "#,
            );
            qb.push_values(chunk, |mut b, item| {
                b.push_bind(&item.order_id);
                b.push_bind(&item.product_id);
                b.push_bind(&item.quantity);
            });
            qb.push(r#" ON DUPLICATE KEY UPDATE `quantity` = VALUES(`quantity`)"#);
            let result = qb.build().execute(&mut **executor).await?;
            total_affected += result.rows_affected();
        }
        Ok(total_affected)
    }

    pub async fn update_unchecked_by_order_id_and_product_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::MySql>,
    >(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        let query_str =
            r#"UPDATE `order_items` SET `quantity` = ? WHERE `order_id` = ? AND `product_id` = ?"#;
        let mut query = sqlx::query::<sqlx::MySql>(query_str);
        query = query.bind(&self.quantity);
        query = query.bind(&self.order_id);
        query = query.bind(&self.product_id);
        let result = query.execute(executor).await?;
        Ok(result.rows_affected())
    }

    pub async fn update_by_order_id_and_product_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::MySql>,
    >(
        &self,
        executor: E,
    ) -> sqlx::Result<u64> {
        if let Err(e) = self.validate() {
            return Err(sqlx::Error::Protocol(e.join(", ").into()));
        }
        self.update_unchecked_by_order_id_and_product_id(executor)
            .await
    }

    pub async fn delete_by_order_id_and_product_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::MySql>,
    >(
        executor: E,
        order_id: i64,
        product_id: i64,
    ) -> sqlx::Result<u64> {
        let query = r#"DELETE FROM `order_items` WHERE `order_id` = ? AND `product_id` = ?"#;
        let result = sqlx::query::<sqlx::MySql>(query)
            .bind(order_id)
            .bind(product_id)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    #[allow(unused_assignments, unused_comparisons, unused_mut, unused_variables)]
    pub async fn update_partial_by_order_id_and_product_id<
        'e,
        E: sqlx::Executor<'e, Database = sqlx::MySql>,
    >(
        executor: E,
        order_id: i64,
        product_id: i64,
        patch: &OrderItemsPatch,
    ) -> sqlx::Result<u64> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(val) = &patch.quantity {
            if (*val as i128) < (0 as i128) {
                errors.push("quantity: minimum value '0' not met".into());
            }
            if (*val as i128) > (2147483647 as i128) {
                errors.push("quantity: exceeds max_value '2147483647'".into());
            }
        }
        if !errors.is_empty() {
            return Err(sqlx::Error::Protocol(errors.join(", ").into()));
        }
        let mut qb: sqlx::QueryBuilder<sqlx::MySql> =
            sqlx::QueryBuilder::new("UPDATE `order_items` SET ");
        let mut first = true;
        if let Some(val) = &patch.quantity {
            if !first {
                qb.push(", ");
            }
            qb.push("`quantity` = ");
            qb.push_bind(val.clone());
            first = false;
        }
        if first {
            return Ok(0);
        }
        qb.push(" WHERE `order_id` = ");
        qb.push_bind(order_id);
        qb.push(" AND `product_id` = ");
        qb.push_bind(product_id);
        let result = qb.build().execute(executor).await?;
        Ok(result.rows_affected())
    }
}

#[allow(clippy::all)]
#[derive(Debug, Clone, Default)]
pub struct OrderItemsPatch {
    pub quantity: Option<i32>,
}
