use crate::{AppError, AppEvent, AppStore, store::AppStorePersistence};

impl AppStore {
    pub async fn load(&self) -> Result<(), AppError> {
        match &self.persistence {
            AppStorePersistence::Database(_) => self.load_from_database().await,
            AppStorePersistence::Filesystem(_) => self.load_from_filesystem().await,
            AppStorePersistence::None => Ok(()),
        }
    }

    pub async fn update(&self, event: AppEvent) -> Result<(), AppError> {
        match &self.persistence {
            AppStorePersistence::Database(_) => self.update_in_database(event).await,
            AppStorePersistence::Filesystem(_) => self.update_in_filesystem(event).await,
            AppStorePersistence::None => {
                let mut data = self.data.write();
                AppStore::apply(event, &mut data);

                Ok(())
            }
        }
    }
}
