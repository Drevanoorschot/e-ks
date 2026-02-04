use crate::{AppError, AppEvent, AppStore, constants::DEFAULT_STREAM_ID};

#[allow(unused)]
pub struct DatabaseEvent {
    pub event_id: i64,
    pub payload: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AppStore {
    pub async fn load(&self) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        match self.catch_up(&mut tx).await {
            Ok(_) => {}
            Err(e) => {
                tx.rollback().await?;

                return Err(e);
            }
        }

        Ok(())

    }

    async fn catch_up(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<usize, AppError> {
        let last_id = self.get_last_event_id()?;

        let row = sqlx::query!(
            r#"SELECT last_event_id
            FROM streams
            WHERE stream_id = $1
            FOR UPDATE"#,
            DEFAULT_STREAM_ID
        )
        .fetch_one(&mut **tx)
        .await?;

        let stream_last_id = row.last_event_id as usize;

        let missing = sqlx::query_as!(
            DatabaseEvent,
            r#"
            SELECT event_id, payload, created_at
            FROM events
            WHERE stream_id = $1 AND event_id > $2
            ORDER BY event_id ASC
            "#,
            DEFAULT_STREAM_ID,
            last_id as i64
        )
        .fetch_all(&mut **tx)
        .await?;

        let mut data = self.data.write().map_err(|_| AppError::InternalServerError)?;
        for event in missing {
            let app_event: AppEvent = serde_json::from_value(event.payload).map_err(|_| AppError::InternalServerError)?;
            AppStore::apply(&app_event, &mut data);
        }

        Ok(stream_last_id)
    }

    async fn append_once(&self, next_id: usize, event: &AppEvent, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(), AppError> {
        let new_payload = serde_json::to_value(event).map_err(|_| AppError::InternalServerError)?;

        sqlx::query!(
            r#"INSERT INTO events (stream_id, event_id, created_at, payload)
            VALUES ($1, $2, $3, $4)"#,
            DEFAULT_STREAM_ID,
            next_id as i64,
            chrono::Utc::now(),
            new_payload
        )
        .execute(&mut **tx)
        .await?;

        // 4) Update stream counter.
        sqlx::query!(
            r#"UPDATE streams SET last_event_id = $2 WHERE stream_id = $1"#,
            DEFAULT_STREAM_ID,
            next_id as i64
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn update(
        &self,
        event: AppEvent,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        let last_id = match self.catch_up(&mut tx).await {
            Ok(id) => id,
            Err(e) => {
                tx.rollback().await?;

                return Err(e);
            }
        };

        let next_id = last_id + 1;

        match self.append_once(next_id, &event, &mut tx).await {
            Ok(_) => {}
            Err(e) => {
                tx.rollback().await?;

                return Err(e);
            }
        }

        tx.commit().await?;

        let mut data = self.data.write().map_err(|_| AppError::InternalServerError)?;
        AppStore::apply(&event, &mut data);
        data.last_event_id = next_id;

        Ok(())
    }
}
