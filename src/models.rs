use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct FeatureFlag {
    pub key: String,
    pub name: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Override {
    pub flag_key: String,
    pub user_id: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateFlagRequest {
    pub key: String,
    pub name: String,
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct ToggleRequest {
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct FlagResponse {
    pub key: String,
    pub name: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct OverrideResponse {
    pub flag_key: String,
    pub user_id: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct EvaluationResponse {
    pub flag_key: String,
    pub user_id: String,
    pub enabled: bool,
    pub reason: String,
}
