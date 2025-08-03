use super::utils::verify_secret_key;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use goose::system_prompts::{SystemPrompt, SystemPromptManager};
use http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SystemPromptsResponse {
    pub prompts: Vec<SystemPrompt>,
}

#[derive(Serialize, ToSchema)]
pub struct SystemPromptResponse {
    pub prompt: SystemPrompt,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateSystemPromptRequest {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub model_specific: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateSystemPromptRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub model_specific: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct SetDefaultRequest {
    pub id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SearchPromptsRequest {
    pub tags: Vec<String>,
}

/// Get all system prompts
#[utoipa::path(
    get,
    path = "/system-prompts",
    responses(
        (status = 200, description = "List of system prompts", body = SystemPromptsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_system_prompts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<SystemPromptsResponse>, StatusCode> {
    verify_secret_key(&headers, &state)?;

    let manager = SystemPromptManager::new();
    manager.initialize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let prompts = manager.list_prompts().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SystemPromptsResponse { prompts }))
}

/// Get a specific system prompt by ID
#[utoipa::path(
    get,
    path = "/system-prompts/{id}",
    responses(
        (status = 200, description = "System prompt details", body = SystemPromptResponse),
        (status = 404, description = "System prompt not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_system_prompt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<SystemPromptResponse>, StatusCode> {
    verify_secret_key(&headers, &state)?;

    let manager = SystemPromptManager::new();
    manager.initialize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Try to find by ID first, then by name
    let prompt = manager.get_prompt(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .or_else(|| {
            manager.get_prompt_by_name(&id)
                .unwrap_or(None)
        })
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(SystemPromptResponse { prompt }))
}

/// Create a new system prompt
#[utoipa::path(
    post,
    path = "/system-prompts",
    request_body = CreateSystemPromptRequest,
    responses(
        (status = 201, description = "System prompt created successfully", body = SystemPromptResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_system_prompt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<CreateSystemPromptRequest>,
) -> Result<Json<SystemPromptResponse>, StatusCode> {
    verify_secret_key(&headers, &state)?;

    let manager = SystemPromptManager::new();
    manager.initialize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut prompt = SystemPrompt::new(request.name, request.content);

    if let Some(description) = request.description {
        prompt = prompt.with_description(description);
    }

    if let Some(tags) = request.tags {
        prompt = prompt.with_tags(tags);
    }

    if let Some(model) = request.model_specific {
        prompt = prompt.with_model_specific(model);
    }

    if request.is_default.unwrap_or(false) {
        prompt = prompt.set_as_default();
    }

    let created_prompt = manager.create_prompt(prompt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SystemPromptResponse { prompt: created_prompt }))
}

/// Update an existing system prompt
#[utoipa::path(
    put,
    path = "/system-prompts/{id}",
    request_body = UpdateSystemPromptRequest,
    responses(
        (status = 200, description = "System prompt updated successfully", body = SystemPromptResponse),
        (status = 404, description = "System prompt not found"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_system_prompt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<UpdateSystemPromptRequest>,
) -> Result<Json<SystemPromptResponse>, StatusCode> {
    verify_secret_key(&headers, &state)?;

    let manager = SystemPromptManager::new();
    manager.initialize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut prompt = manager.get_prompt(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = request.name {
        prompt.name = name;
    }

    if let Some(description) = request.description {
        prompt.description = Some(description);
    }

    if let Some(content) = request.content {
        prompt.update_content(content);
    }

    if let Some(tags) = request.tags {
        prompt.tags = tags;
    }

    if let Some(model) = request.model_specific {
        prompt.model_specific = Some(model);
    }

    let updated_prompt = manager.update_prompt(&id, prompt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SystemPromptResponse { prompt: updated_prompt }))
}

/// Delete a system prompt
#[utoipa::path(
    delete,
    path = "/system-prompts/{id}",
    responses(
        (status = 200, description = "System prompt deleted successfully"),
        (status = 404, description = "System prompt not found"),
        (status = 400, description = "Cannot delete default prompt"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_system_prompt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<String>, StatusCode> {
    verify_secret_key(&headers, &state)?;

    let manager = SystemPromptManager::new();
    manager.initialize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    manager.delete_prompt(&id)
        .map_err(|e| {
            if e.to_string().contains("Cannot delete the default") {
                StatusCode::BAD_REQUEST
            } else if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(Json("System prompt deleted successfully".to_string()))
}

/// Set a system prompt as default
#[utoipa::path(
    post,
    path = "/system-prompts/{id}/set-default",
    responses(
        (status = 200, description = "Default system prompt set successfully"),
        (status = 404, description = "System prompt not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn set_default_system_prompt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<String>, StatusCode> {
    verify_secret_key(&headers, &state)?;

    let manager = SystemPromptManager::new();
    manager.initialize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    manager.set_default_prompt(&id)
        .map_err(|e| {
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(Json("Default system prompt set successfully".to_string()))
}

/// Search system prompts by tags
#[utoipa::path(
    post,
    path = "/system-prompts/search",
    request_body = SearchPromptsRequest,
    responses(
        (status = 200, description = "Matching system prompts", body = SystemPromptsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_system_prompts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<SearchPromptsRequest>,
) -> Result<Json<SystemPromptsResponse>, StatusCode> {
    verify_secret_key(&headers, &state)?;

    let manager = SystemPromptManager::new();
    manager.initialize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let prompts = manager.search_by_tags(&request.tags)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SystemPromptsResponse { prompts }))
}

/// Get the default system prompt
#[utoipa::path(
    get,
    path = "/system-prompts/default",
    responses(
        (status = 200, description = "Default system prompt", body = SystemPromptResponse),
        (status = 404, description = "No default system prompt found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_default_system_prompt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<SystemPromptResponse>, StatusCode> {
    verify_secret_key(&headers, &state)?;

    let manager = SystemPromptManager::new();
    manager.initialize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let prompt = manager.get_default_prompt()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(SystemPromptResponse { prompt }))
}

/// Configure system prompt routes
pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/system-prompts", get(list_system_prompts))
        .route("/system-prompts", post(create_system_prompt))
        .route("/system-prompts/search", post(search_system_prompts))
        .route("/system-prompts/default", get(get_default_system_prompt))
        .route("/system-prompts/{id}", get(get_system_prompt))
        .route("/system-prompts/{id}", put(update_system_prompt))
        .route("/system-prompts/{id}", delete(delete_system_prompt))
        .route("/system-prompts/{id}/set-default", post(set_default_system_prompt))
        .with_state(state)
}