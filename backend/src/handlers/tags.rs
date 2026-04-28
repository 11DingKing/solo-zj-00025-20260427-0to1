use crate::cache::CacheService;
use crate::db::AppState;
use crate::errors::{AppError, AppResult};
use crate::handlers::boards::{verify_board_access, verify_board_permission};
use crate::handlers::cards::get_card_board_id;
use crate::middleware::auth::CurrentUser;
use crate::models::{BoardIdNameRow, BoardIdRow, IdNameBoardIdRow, IdRow, NameRow, TitleRow, CreateTagRequest, EntityType, Tag, UpdateTagRequest};
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

fn validate_color(color: &str) -> Result<(), AppError> {
    let re = regex::Regex::new(r"^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$").unwrap();
    if re.is_match(color) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!("Invalid color format: {}", color)))
    }
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
    validate_color(&req.color)?;
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

    sqlx::query(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(board_id)
    .bind(current_user.user.id)
    .bind("created")
    .bind(EntityType::Tag.as_str())
    .bind(tag.id)
    .bind(serde_json::json!({ "name": tag.name, "color": tag.color }))
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

    if let Some(ref color) = req.color {
        validate_color(color)?;
    }

    let tag = sqlx::query_as::<_, BoardIdRow>(
        r#"SELECT board_id FROM tags WHERE id = $1"#,
    )
    .bind(tag_id)
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

    sqlx::query(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(tag.board_id)
    .bind(current_user.user.id)
    .bind("updated")
    .bind(EntityType::Tag.as_str())
    .bind(tag.id)
    .bind(serde_json::json!({ "name": req.name, "color": req.color }))
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
    let tag = sqlx::query_as::<_, BoardIdNameRow>(
        r#"SELECT board_id, name FROM tags WHERE id = $1"#,
    )
    .bind(tag_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Tag not found".to_string()))?;

    verify_board_permission(&state, current_user.user.id, tag.board_id, |r| r.can_edit_board()).await?;

    sqlx::query(r#"DELETE FROM tags WHERE id = $1"#)
        .bind(tag_id)
        .execute(&state.db)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(tag.board_id)
    .bind(current_user.user.id)
    .bind("deleted")
    .bind(EntityType::Tag.as_str())
    .bind(tag_id)
    .bind(serde_json::json!({ "name": tag.name }))
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

    let tag = sqlx::query_as::<_, IdNameBoardIdRow>(
        r#"SELECT id, name, board_id FROM tags WHERE id = $1"#,
    )
    .bind(tag_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Tag not found".to_string()))?;

    if tag.board_id != board_id {
        return Err(AppError::BadRequest("Tag does not belong to this board".to_string()));
    }

    let existing = sqlx::query_as::<_, IdRow>(
        r#"SELECT id FROM card_tags WHERE card_id = $1 AND tag_id = $2"#,
    )
    .bind(card_id)
    .bind(tag_id)
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Ok(Json(serde_json::json!({ "message": "Tag already added" })));
    }

    sqlx::query(
        r#"
        INSERT INTO card_tags (card_id, tag_id)
        VALUES ($1, $2)
        "#,
    )
    .bind(card_id)
    .bind(tag_id)
    .execute(&state.db)
    .await?;

    let card = sqlx::query_as::<_, TitleRow>(
        r#"SELECT title FROM cards WHERE id = $1"#,
    )
    .bind(card_id)
    .fetch_optional(&state.db)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(board_id)
    .bind(current_user.user.id)
    .bind("tag_added")
    .bind(EntityType::Card.as_str())
    .bind(card_id)
    .bind(serde_json::json!({ "card_title": card.map(|c| c.title), "tag_name": tag.name }))
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

    let tag = sqlx::query_as::<_, NameRow>(
        r#"SELECT name FROM tags WHERE id = $1"#,
    )
    .bind(tag_id)
    .fetch_optional(&state.db)
    .await?;

    let result = sqlx::query(
        r#"DELETE FROM card_tags WHERE card_id = $1 AND tag_id = $2"#,
    )
    .bind(card_id)
    .bind(tag_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Tag not found on card".to_string()));
    }

    let card = sqlx::query_as::<_, TitleRow>(
        r#"SELECT title FROM cards WHERE id = $1"#,
    )
    .bind(card_id)
    .fetch_optional(&state.db)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(board_id)
    .bind(current_user.user.id)
    .bind("tag_removed")
    .bind(EntityType::Card.as_str())
    .bind(card_id)
    .bind(serde_json::json!({ "card_title": card.map(|c| c.title), "tag_name": tag.map(|t| t.name) }))
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "message": "Tag removed" })))
}
