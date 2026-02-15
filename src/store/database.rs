use crate::{
    AppError, AppEvent, AppStore, UtcDateTime, constants::DEFAULT_STREAM_ID,
    store::{AppStorePersistence, StoreEvent},
};

#[derive(Debug, sqlx::FromRow)]
struct DbEventRow {
    pub event_id: i64,
    pub payload: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AppStore {
    pub async fn load_from_database(&self) -> Result<(), AppError> {
        let AppStorePersistence::Database(pool) = &self.persistence else {
            return Ok(());
        };

        let mut tx = pool.begin().await?;

        match self.catch_up_database(&mut tx).await {
            Ok(_) => {}
            Err(e) => {
                tx.rollback().await?;

                return Err(e);
            }
        }

        Ok(())
    }

    async fn catch_up_database(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<usize, AppError> {
        let last_id: usize = self.get_last_event_id()?;

        let stream_last_id: i64 = sqlx::query_scalar(
            r#"SELECT last_event_id
            FROM streams
            WHERE stream_id = $1
            FOR UPDATE"#,
        )
        .bind(DEFAULT_STREAM_ID)
        .fetch_one(&mut **tx)
        .await?;

        let missing: Vec<DbEventRow> = sqlx::query_as::<_, DbEventRow>(
            r#"
            SELECT event_id, payload, created_at
            FROM events
            WHERE stream_id = $1 AND event_id > $2
            ORDER BY event_id ASC
            "#,
        )
        .bind(DEFAULT_STREAM_ID)
        .bind(last_id as i64)
        .fetch_all(&mut **tx)
        .await?;

        let mut data = self.data.write();

        for row in missing {
            if data.last_event_id >= row.event_id as usize {
                // This can happen if another instance of the application processed events concurrently
                // and updated the store before this instance could acquire the write lock. In that case,
                // the store is already up-to-date and we can skip applying the event again.
                continue;
            }

            let payload = match self.decrypt_event_payload(&row.payload) {
                Ok(payload) => payload,
                Err(decrypt_error) => match postcard::from_bytes::<AppEvent>(&row.payload) {
                    Ok(payload) => {
                        tracing::warn!(
                            "Loaded legacy unencrypted event payload: {decrypt_error:?}"
                        );
                        payload
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to decrypt or deserialize event payload: {decrypt_error:?}, {e:?}"
                        );
                        continue;
                    }
                },
            };

            let event = StoreEvent {
                event_id: row.event_id,
                created_at: UtcDateTime::from(row.created_at),
                payload,
            };
            AppStore::apply(event.payload, &mut data);
            data.last_event_id = event.event_id as usize;
        }

        Ok(stream_last_id as usize)
    }

    async fn append_to_database(
        &self,
        event: &StoreEvent,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), AppError> {
        let new_payload = self.encrypt_event_payload(&event.payload)?;
        let created_at: chrono::DateTime<chrono::Utc> = event.created_at.into();

        sqlx::query(
            r#"INSERT INTO events (stream_id, event_id, created_at, payload)
            VALUES ($1, $2, $3, $4)"#,
        )
        .bind(DEFAULT_STREAM_ID)
        .bind(event.event_id)
        .bind(created_at)
        .bind(new_payload)
        .execute(&mut **tx)
        .await?;

        sqlx::query(r#"UPDATE streams SET last_event_id = $2 WHERE stream_id = $1"#)
            .bind(DEFAULT_STREAM_ID)
            .bind(event.event_id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }

    pub async fn update_in_database(&self, event: AppEvent) -> Result<(), AppError> {
        let AppStorePersistence::Database(pool) = &self.persistence else {
            return Ok(());
        };

        sqlx::query(
            r#"INSERT INTO streams (stream_id, last_event_id)
            VALUES ($1, 0)
            ON CONFLICT (stream_id) DO NOTHING"#,
        )
        .bind(crate::common::constants::DEFAULT_STREAM_ID)
        .execute(pool)
        .await?;

        let mut tx = pool.begin().await?;

        let last_id = match self.catch_up_database(&mut tx).await {
            Ok(id) => id,
            Err(e) => {
                tx.rollback().await?;

                return Err(e);
            }
        };

        let next_id = last_id + 1;
        let store_event = StoreEvent {
            event_id: next_id as i64,
            created_at: UtcDateTime::now(),
            payload: event.clone(),
        };

        match self.append_to_database(&store_event, &mut tx).await {
            Ok(_) => {}
            Err(e) => {
                tx.rollback().await?;

                return Err(e);
            }
        }

        tx.commit().await?;

        let mut data = self.data.write();

        if data.last_event_id >= next_id {
            // This can happen if another instance of the application processed events concurrently
            // and updated the store before this instance could acquire the write lock. In that case,
            // the store is already up-to-date and we can skip applying the event again.
            return Ok(());
        }

        AppStore::apply(event, &mut data);
        data.last_event_id = next_id;

        Ok(())
    }

    #[cfg(feature = "fixtures")]
    pub async fn clear(&self) -> Result<(), AppError> {
        let AppStorePersistence::Database(pool) = &self.persistence else {
            return Ok(());
        };

        sqlx::migrate!().run(pool).await.map_err(|e| {
            tracing::error!("Failed to run migrations: {e:?}");
            AppError::InternalServerError
        })?;

        sqlx::query("TRUNCATE TABLE streams CASCADE")
            .execute(pool)
            .await?;
        sqlx::query("TRUNCATE TABLE events CASCADE")
            .execute(pool)
            .await?;

        sqlx::query(
            r#"INSERT INTO streams (stream_id, last_event_id)
            VALUES ($1, 0)
            ON CONFLICT (stream_id) DO NOTHING"#,
        )
        .bind(crate::common::constants::DEFAULT_STREAM_ID)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{persons::PersonId, store::test_event_crypto, test_utils::sample_person};
    use chrono::Utc;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn update_persists_and_load_replays(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool.clone(), test_event_crypto());
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let loaded = store.get_person(person_id)?;
        assert_eq!(loaded.id, person_id);

        let fresh_store = AppStore::new(pool, test_event_crypto());
        fresh_store.load().await?;

        let reloaded = fresh_store.get_person(person_id)?;
        assert_eq!(reloaded.id, person_id);

        Ok(())
    }

    #[sqlx::test]
    async fn load_skips_invalid_payloads(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool.clone(), test_event_crypto());
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let invalid_payload = vec![0_u8, 1, 2, 3];
        sqlx::query(
            r#"INSERT INTO events (stream_id, event_id, created_at, payload)
            VALUES ($1, $2, $3, $4)"#,
        )
        .bind(DEFAULT_STREAM_ID)
        .bind(2_i64)
        .bind(Utc::now())
        .bind(invalid_payload)
        .execute(&pool)
        .await?;

        sqlx::query(r#"UPDATE streams SET last_event_id = $2 WHERE stream_id = $1"#)
            .bind(DEFAULT_STREAM_ID)
            .bind(2_i64)
            .execute(&pool)
            .await?;

        let fresh_store = AppStore::new(pool, test_event_crypto());
        fresh_store.load().await?;

        let reloaded = fresh_store.get_person(person_id)?;
        assert_eq!(reloaded.id, person_id);

        Ok(())
    }
}
