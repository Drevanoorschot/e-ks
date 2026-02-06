use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InitialEditQuery {
    initial: Option<bool>,
}

impl InitialEditQuery {
    pub fn should_warn(&self) -> bool {
        !self.initial.unwrap_or(false)
    }
}

impl Default for InitialEditQuery {
    fn default() -> Self {
        Self {
            initial: Some(true),
        }
    }
}
