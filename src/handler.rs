use crate::{
    errors::AppError,
    models::{CreateFlagRequest, FeatureFlag, FlagResponse, Override},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use std::sync::Arc;

pub async fn create_flag(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateFlagRequest>,
) -> Result<impl IntoResponse, AppError> {
    let normalized_key = payload.key.trim().to_ascii_lowercase();

    if normalized_key.is_empty() {
        return Err(AppError::BadRequest("Key cannot be empty".to_string()));
    }

    if !normalized_key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::BadRequest(
            "Key can only contain alphanumeric characters, underscores, and hyphens".to_string(),
        ));
    }

    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".to_string()));
    }

    if state.flags.contains_key(&normalized_key) {
        return Err(AppError::Conflict(format!(
            "Feature flag with key '{}' already exists",
            normalized_key
        )));
    }

    let flag = FeatureFlag {
        key: normalized_key.clone(),
        name: payload.name.trim().to_string(),
        enabled: payload.enabled,
        created_at: Utc::now(),
    };

    state.flags.insert(normalized_key.clone(), flag.clone());

    let response = FlagResponse {
        key: flag.key,
        name: flag.name,
        enabled: flag.enabled,
        created_at: flag.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_flag(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let normalized_key = key.trim().to_ascii_lowercase();

    let flag = state.flags.get(&normalized_key).ok_or_else(|| {
        AppError::NotFound(format!(
            "Feature flag with key '{}' not found",
            normalized_key
        ))
    })?;

    let response = FlagResponse {
        key: flag.key.clone(),
        name: flag.name.clone(),
        enabled: flag.enabled,
        created_at: flag.created_at,
    };

    Ok(Json(response))
}
