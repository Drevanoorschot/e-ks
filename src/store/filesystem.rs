use std::path::{Path, PathBuf};

use crate::{
    AppError, AppEvent, AppStore, UtcDateTime, constants::DEFAULT_STREAM_ID,
    store::{AppStorePersistence, StoreEvent},
};
use serde::{Deserialize, Serialize};

impl AppStore {
    pub async fn load_from_filesystem(&self) -> Result<(), AppError> {
        let AppStorePersistence::Filesystem(root) = &self.persistence else {
            return Ok(());
        };

        let stream_path = stream_file_path(root, DEFAULT_STREAM_ID);
        let events = read_stream_events(self, &stream_path).await?;

        let mut data = self.data.write();

        for event in events {
            if data.last_event_id >= event.event_id as usize {
                continue;
            }

            AppStore::apply(event.payload, &mut data);
            data.last_event_id = event.event_id as usize;
        }

        Ok(())
    }

    pub async fn update_in_filesystem(&self, event: AppEvent) -> Result<(), AppError> {
        let AppStorePersistence::Filesystem(root) = &self.persistence else {
            return Ok(());
        };

        tokio::fs::create_dir_all(root)
            .await
            .map_err(AppError::ServerError)?;

        let stream_path = stream_file_path(root, DEFAULT_STREAM_ID);
        let mut events = read_stream_events(self, &stream_path).await?;
        let next_id = events.last().map(|ev| ev.event_id).unwrap_or(0) + 1;

        let store_event = StoreEvent {
            event_id: next_id,
            created_at: UtcDateTime::now(),
            payload: event.clone(),
        };

        append_stream_event(self, &stream_path, &store_event).await?;
        events.push(store_event);

        let mut data = self.data.write();

        if data.last_event_id >= next_id as usize {
            return Ok(());
        }

        AppStore::apply(event, &mut data);
        data.last_event_id = next_id as usize;

        Ok(())
    }
}

fn stream_file_path(root: &Path, stream_id: uuid::Uuid) -> PathBuf {
    root.join(format!("{stream_id}.bin"))
}

async fn read_stream_events(store: &AppStore, path: &Path) -> Result<Vec<StoreEvent>, AppError> {
    let contents = match tokio::fs::read(path).await {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(AppError::ServerError(err)),
    };

    if contents.is_empty() {
        return Ok(Vec::new());
    }

    decode_stream_events(store, &contents)
}

async fn append_stream_event(
    store: &AppStore,
    path: &Path,
    event: &StoreEvent,
) -> Result<(), AppError> {
    let payload = store.encrypt_event_payload(&event.payload)?;
    let record_event = EncryptedStoreEvent {
        version: 1,
        event_id: event.event_id,
        created_at: event.created_at,
        payload,
    };
    let payload = postcard::to_stdvec(&record_event).map_err(|_| AppError::InternalServerError)?;
    let mut record = Vec::with_capacity(4 + payload.len());
    let len: u32 = payload
        .len()
        .try_into()
        .map_err(|_| AppError::InternalServerError)?;
    record.extend_from_slice(&len.to_le_bytes());
    record.extend_from_slice(&payload);

    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(AppError::ServerError)?;

    file.write_all(&record).await.map_err(AppError::ServerError)?;
    file.flush().await.map_err(AppError::ServerError)?;

    Ok(())
}

fn decode_stream_events(store: &AppStore, contents: &[u8]) -> Result<Vec<StoreEvent>, AppError> {
    let mut events = Vec::new();
    let mut cursor = 0usize;

    while cursor < contents.len() {
        if cursor + 4 > contents.len() {
            return Err(AppError::IntegrityViolation);
        }

        let len_bytes = [contents[cursor], contents[cursor + 1], contents[cursor + 2], contents[cursor + 3]];
        let len = u32::from_le_bytes(len_bytes) as usize;
        cursor += 4;

        if cursor + len > contents.len() {
            return Err(AppError::IntegrityViolation);
        }

        let payload = &contents[cursor..cursor + len];
        let event = decode_store_event(store, payload)?;
        events.push(event);
        cursor += len;
    }

    Ok(events)
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedStoreEvent {
    version: u8,
    event_id: i64,
    created_at: UtcDateTime,
    payload: Vec<u8>,
}

fn decode_store_event(store: &AppStore, payload: &[u8]) -> Result<StoreEvent, AppError> {
    let encrypted: Result<EncryptedStoreEvent, _> = postcard::from_bytes(payload);
    if let Ok(encrypted) = encrypted {
        if encrypted.version == 1 {
            let decrypted = store.decrypt_event_payload(&encrypted.payload)?;
            return Ok(StoreEvent {
                event_id: encrypted.event_id,
                created_at: encrypted.created_at,
                payload: decrypted,
            });
        }
    }

    postcard::from_bytes(payload).map_err(|_| AppError::InternalServerError)
}
