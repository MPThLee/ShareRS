use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids;

#[serde_with::serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: ids::FileId,
    pub name: String,
    pub original_name: Option<String>,
    pub mime: Option<String>,
    pub views: u64,
    pub max_views: Option<u64>,
    pub user_id: ids::UserId,
    pub created_at: DateTime<Utc>,
}
