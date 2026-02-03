use sqlx::PgPool;

use crate::{AppError, AppStore};

mod candidate_list;
mod persons;
mod political_groups;

pub async fn load(pool: &PgPool, store: &AppStore) -> Result<(), AppError> {
    clear_database(pool).await?;
    persons::load(store).await?;
    candidate_list::load(store).await?;
    political_groups::load(store).await?;

    Ok(())
}

async fn clear_database(_db: &PgPool) -> Result<(), AppError> {
    // TODO

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{AppStore, fixtures::load};
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_load_all_fixtures(pool: PgPool) {
        let store = AppStore::default();
        load(&pool, &store).await.unwrap();
        let persons = crate::persons::Person::list(
            &store,
            50,
            0,
            &crate::persons::PersonSort::LastName,
            &crate::pagination::SortDirection::Asc,
        );

        assert_eq!(persons.len(), 50);
    }
}
