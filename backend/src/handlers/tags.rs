use crate::cache::CacheService;
use crate::db::AppState;
use crate::errors::{AppError, AppResult};
use crate::handlers::boards::{verify_board_access, verify_board_permission};
use crate::handlers::cards::get_card_board_id;
use crate::middleware::auth::CurrentUser;
use crate::models::{CreateTagRequest, EntityType, Tag, UpdateTagRequest};
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

fn tags_cache_key(board_id: Uuid) -> String {
    format!("board:{}:tags", board_id)
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(board_id): Path<Uuid>,
) -> AppResult<Json<Vec<Tag>>> {
    verify_board_access(&state, current_user.user.id, board_id).await?;

    let cache = CacheService::new(state.redis.clone());
    let cache_key = tags_cache_key(board_id);

    if let Some(cached) = cache.get::<Vec<Tag>>(&cache_key).await? {
        return Ok(Json(cached));
    }

    let tags = sqlx::query_as::<_, Tag>(
        r#"
        SELECT id, board_id, name, color, created_at 
        FROM tags 
        WHERE board_id = $1 
        ORDER BY created_at
        "#,
    )
    .bind(board_id)
    .fetch_all(&state.db)
    .await?;

    cache.set(&cache_key, &tags, 300).await?;

    Ok(Json(tags))
}

pub async fn create_tag(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(board_id): Path<Uuid>,
    Json(req): Json<CreateTagRequest>,
) -> AppResult<Json<Tag>> {
    req.validate()?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_board()).await?;

    let tag = sqlx::query_as::<_, Tag>(
        r#"
        INSERT INTO tags (board_id, name, color)
        VALUES ($1, $2, $3)
        RETURNING id, board_id, name, color, created_at
        "#,
    )
    .bind(board_id)
    .bind(&req.name)
    .bind(&req.color)
    .fetch_one(&state.db)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        board_id,
        current_user.user.id,
        "created",
        EntityType::Tag.as_str(),
        tag.id,
        serde_json::json!({ "name": tag.name, "color": tag.color })
    )
    .execute(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&tags_cache_key(board_id)).await?;

    Ok(Json(tag))
}

pub async fn update_tag(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(tag_id): Path<Uuid>,
    Json(req): Json<UpdateTagRequest>,
) -> AppResult<Json<Tag>> {
    req.validate()?;

    let tag = sqlx::query!(
        r#"SELECT board_id FROM tags WHERE id = $1"#,
        tag_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Tag not found".to_string()))?;

    verify_board_permission(&state, current_user.user.id, tag.board_id, |r| r.can_edit_board()).await?;

    let tag = sqlx::query_as::<_, Tag>(
        r#"
        UPDATE tags
        SET 
            name = COALESCE($1, name),
            color = COALESCE($2, color)
        WHERE id = $3
        RETURNING id, board_id, name, color, created_at
        "#,
    )
    .bind(&req.name)
    .bind(&req.color)
    .bind(tag_id)
    .fetch_one(&state.db)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        tag.board_id,
        current_user.user.id,
        "updated",
        EntityType::Tag.as_str(),
        tag.id,
        serde_json::json!({ "name": req.name, "color": req.color })
    )
    .execute(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&tags_cache_key(tag.board_id)).await?;

    Ok(Json(tag))
}

pub async fn delete_tag(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(tag_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let tag = sqlx::query!(
        r#"SELECT board_id, name FROM tags WHERE id = $1"#,
        tag_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Tag not found".to_string()))?;

    verify_board_permission(&state, current_user.user.id, tag.board_id, |r| r.can_edit_board()).await?;

    sqlx::query!(r#"DELETE FROM tags WHERE id = $1"#, tag_id)
        .execute(&state.db)
        .await?;

    sqlx::query!(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        tag.board_id,
        current_user.user.id,
        "deleted",
        EntityType::Tag.as_str(),
        tag_id,
        serde_json::json!({ "name": tag.name })
    )
    .execute(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&tags_cache_key(tag.board_id)).await?;

    Ok(Json(serde_json::json!({ "message": "Tag deleted" })))
}

pub async fn add_tag_to_card(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path((card_id, tag_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    let board_id = get_card_board_id(&state, card_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let tag = sqlx::query!(
        r#"SELECT id, name, board_id FROM tags WHERE id = $1"#,
        tag_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Tag not found".to_string()))?;

    if tag.board_id != board_id {
        return Err(AppError::BadRequest("Tag does not belong to this board".to_string()));
    }

    let existing = sqlx::query!(
        r#"SELECT id FROM card_tags WHERE card_id = $1 AND tag_id = $2"#,
        card_id,
        tag_id
    )
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Ok(Json(serde_json::json!({ "message": "Tag already added" })));
    }

    sqlx::query!(
        r#"
        INSERT INTO card_tags (card_id, tag_id)
        VALUES ($1, $2)
        "#,
        card_id,
        tag_id
    )
    .execute(&state.db)
    .await?;

    let card = sqlx::query!(
        r#"SELECT title FROM cards WHERE id = $1"#,
        card_id
    )
    .fetch_optional(&state.db)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        board_id,
        current_user.user.id,
        "tag_added",
        EntityType::Card.as_str(),
        card_id,
        serde_json::json!({ "card_title": card.map(|c| c.title), "tag_name": tag.name })
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "message": "Tag added" })))
}

pub async fn remove_tag_from_card(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path((card_id, tag_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    let board_id = get_card_board_id(&state, card_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let tag = sqlx::query!(
        r#"SELECT name FROM tags WHERE id = $1"#,
        tag_id
    )
    .fetch_optional(&state.db)
    .await?;

    let result = sqlx::query!(
        r#"DELETE FROM card_tags WHERE card_id = $1 AND tag_id = $2"#,
        card_id,
        tag_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Tag not found on card".to_string()));
    }

    let card = sqlx::query!(
        r#"SELECT title FROM cards WHERE id = $1"#,
        card_id
    )
    .fetch_optional(&state.db)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        board_id,
        current_user.user.id,
        "tag_removed",
        EntityType::Card.as_str(),
        card_id,
        serde_json::json!({ "card_title": card.map(|c| c.title), "tag_name": tag.map(|t| t.name) })
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "message": "Tag removed" })))
}
