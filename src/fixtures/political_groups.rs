use chrono::Utc;
use uuid::Uuid;

use crate::{
    AppError, AppStore,
    political_groups::{
        AuthorisedAgent, AuthorisedAgentId, ListSubmitter, ListSubmitterId, PoliticalGroup,
        PoliticalGroupId,
    },
};

pub async fn load(store: &AppStore) -> Result<(), AppError> {
    let political_group_id: PoliticalGroupId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_political_group").into();

    let agent_1_id: AuthorisedAgentId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_authorised_agent_1").into();
    let agent_2_id: AuthorisedAgentId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_authorised_agent_2").into();

    let submitter_1_id: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_list_submitter_1").into();
    let submitter_2_id: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_list_submitter_2").into();

    let political_group = PoliticalGroup {
        id: political_group_id,
        long_list_allowed: None,
        legal_name: Some("Kiesraad Demo Partij".to_string()),
        display_name: Some("Kiesraad Demo".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let _political_group = political_group.create(store).await?;

    AuthorisedAgent {
        id: agent_1_id,
        last_name: "Jansen".to_string(),
        last_name_prefix: Some("de".to_string()),
        initials: "A.B.".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
    .create(store, political_group_id)
    .await?;

    AuthorisedAgent {
        id: agent_2_id,
        last_name: "Visser".to_string(),
        last_name_prefix: None,
        initials: "C.D.".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
    .create(store, political_group_id)
    .await?;

    ListSubmitter {
        id: submitter_1_id,
        last_name: "Bos".to_string(),
        last_name_prefix: None,
        initials: "E.F.".to_string(),
        locality: Some("Rotterdam".to_string()),
        postal_code: Some("3011 CC".to_string()),
        house_number: Some("5".to_string()),
        house_number_addition: Some("B".to_string()),
        street_name: Some("Coolsingel".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
    .create(store, political_group_id)
    .await?;

    ListSubmitter {
        id: submitter_2_id,
        last_name: "Smit".to_string(),
        last_name_prefix: Some("van".to_string()),
        initials: "G.H.".to_string(),
        locality: Some("Den Haag".to_string()),
        postal_code: Some("2511 DD".to_string()),
        house_number: Some("18".to_string()),
        house_number_addition: None,
        street_name: Some("Spui".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
    .create(store, political_group_id)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load() {
        let store = AppStore::default();
        load(&store).await.unwrap();

        let group = store.get_political_group();

        let list_submitters = PoliticalGroup::list_submitters(&store, group.id).unwrap();
        assert_eq!(list_submitters.len(), 2);

        let authorised_count = PoliticalGroup::list_authorised_agents(&store, group.id)
            .unwrap()
            .len();
        assert_eq!(authorised_count, 2);
    }
}
