use crate::{
    errors::AppError,
    models::{
        CreateFlagRequest, EvaluationResponse, FeatureFlag, FlagResponse, Override,
        OverrideResponse, ToggleRequest,
    },
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use std::collections::HashMap;
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

pub async fn toggle_flag(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(payload): Json<ToggleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let normalized_key = key.trim().to_ascii_lowercase();

    let mut flag = state.flags.get_mut(&normalized_key).ok_or_else(|| {
        AppError::NotFound(format!(
            "Feature flag with key '{}' not found",
            normalized_key
        ))
    })?;

    flag.enabled = payload.enabled;

    let response = FlagResponse {
        key: flag.key.clone(),
        name: flag.name.clone(),
        enabled: flag.enabled,
        created_at: flag.created_at,
    };

    Ok(Json(response))
}

pub async fn set_override(
    State(state): State<Arc<AppState>>,
    Path((key, user_id)): Path<(String, String)>,
    Json(payload): Json<ToggleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let normalized_key = key.trim().to_ascii_lowercase();
    let normalized_user_id = user_id.trim().to_string();

    if normalized_user_id.is_empty() {
        return Err(AppError::BadRequest("User ID cannot be empty".to_string()));
    }
    if !state.flags.contains_key(&normalized_key) {
        return Err(AppError::NotFound(format!(
            "Feature flag with key '{}' not found",
            normalized_key
        )));
    }

    let override_entry = Override {
        flag_key: normalized_key.clone(),
        user_id: normalized_user_id.clone(),
        enabled: payload.enabled,
        created_at: Utc::now(),
    };

    state
        .overrides
        .entry(normalized_key)
        .or_insert_with(HashMap::new)
        .insert(normalized_user_id, override_entry.clone());

    let response = OverrideResponse {
        flag_key: override_entry.flag_key,
        user_id: override_entry.user_id,
        enabled: override_entry.enabled,
        created_at: override_entry.created_at,
    };

    Ok(Json(response))
}

pub async fn evaluate_flag(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let normalized_key = key.trim().to_ascii_lowercase();

    let user_id = params
        .get("user_id")
        .ok_or_else(|| AppError::BadRequest("user_id is required".to_string()))?
        .trim()
        .to_string();

    if user_id.is_empty() {
        return Err(AppError::BadRequest("user_id cannot be empty".to_string()));
    }

    let result = evaluate(&normalized_key, &user_id, &state)?;

    Ok(Json(result))
}

pub fn evaluate(
    key: &str,
    user_id: &str,
    state: &AppState,
) -> Result<EvaluationResponse, AppError> {
    let flag = state
        .flags
        .get(key)
        .ok_or_else(|| AppError::NotFound(format!("Feature flag with key '{}' not found", key)))?;

    if let Some(overrides) = state.overrides.get(key)
        && let Some(override_entry) = overrides.get(user_id) {
            return Ok(EvaluationResponse {
                flag_key: key.to_string(),
                user_id: user_id.to_string(),
                enabled: override_entry.enabled,
                reason: "user_override".to_string(),
            });
        }

    Ok(EvaluationResponse {
        flag_key: key.to_string(),
        user_id: user_id.to_string(),
        enabled: flag.enabled,
        reason: "global".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use chrono::Utc;

    fn make_state() -> AppState {
        AppState::new()
    }

    fn make_flag(key: &str, enabled: bool) -> FeatureFlag {
        FeatureFlag {
            key: key.to_string(),
            name: "Test Flag".to_string(),
            enabled,
            created_at: Utc::now(),
        }
    }

    fn make_override(flag_key: &str, user_id: &str, enabled: bool) -> Override {
        Override {
            flag_key: flag_key.to_string(),
            user_id: user_id.to_string(),
            enabled,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_evaluate_returns_global_when_no_override() {
        let state = make_state();
        state
            .flags
            .insert("my-flag".into(), make_flag("my-flag", true));

        let result = evaluate("my-flag", "user-1", &state).unwrap();

        assert!(result.enabled);
        assert_eq!(result.reason, "global");
    }

    #[test]
    fn test_evaluate_returns_override_when_exists() {
        let state = make_state();
        state
            .flags
            .insert("my-flag".into(), make_flag("my-flag", false));

        let mut map = HashMap::new();
        map.insert(
            "user-1".to_string(),
            make_override("my-flag", "user-1", true),
        );
        state.overrides.insert("my-flag".into(), map);

        let result = evaluate("my-flag", "user-1", &state).unwrap();

        assert_eq!(result.enabled, true);
        assert_eq!(result.reason, "user_override");
    }

    #[test]
    fn test_evaluate_override_wins_over_global() {
        let state = make_state();
        state
            .flags
            .insert("my-flag".into(), make_flag("my-flag", true));

        let mut map = HashMap::new();
        map.insert(
            "user-1".to_string(),
            make_override("my-flag", "user-1", false),
        );
        state.overrides.insert("my-flag".into(), map);

        let result = evaluate("my-flag", "user-1", &state).unwrap();

        assert_eq!(result.enabled, false);
        assert_eq!(result.reason, "user_override");
    }

    #[test]
    fn test_evaluate_returns_error_when_flag_not_found() {
        let state = make_state();

        let result = evaluate("missing-flag", "user-1", &state);

        assert!(result.is_err());
    }
}
