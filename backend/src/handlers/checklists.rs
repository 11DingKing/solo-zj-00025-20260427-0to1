use crate::db::AppState;
use crate::errors::{AppError, AppResult};
use crate::handlers::boards::{verify_board_access, verify_board_permission};
use crate::handlers::cards::get_card_board_id;
use crate::middleware::auth::CurrentUser;
use crate::models::{
    BoardIdRow, Checklist, ChecklistCardIdTitleRow, ChecklistItemStateRow, MaxPositionRow, TitleRow,
    ChecklistItem, CreateChecklistItemRequest, CreateChecklistRequest, EntityType,
    UpdateChecklistItemRequest, UpdateChecklistRequest,
};
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

async fn get_checklist_board_id(state: &Arc<AppState>, checklist_id: Uuid) -> AppResult<Uuid> {
    let result = sqlx::query_as::<_, BoardIdRow>(
        r#"
        SELECT c.board_id 
        FROM checklists cl
        JOIN cards ca ON cl.card_id = ca.id
        JOIN columns c ON ca.column_id = c.id
        WHERE cl.id = $1
        "#,
    )
    .bind(checklist_id)
    .fetch_optional(&state.db)
    .await?;

    match result {
        Some(row) => Ok(row.board_id),
        None => Err(AppError::NotFound("Checklist not found".to_string())),
    }
}

async fn get_checklist_item_board_id(state: &Arc<AppState>, item_id: Uuid) -> AppResult<Uuid> {
    let result = sqlx::query_as::<_, BoardIdRow>(
        r#"
        SELECT c.board_id 
        FROM checklist_items ci
        JOIN checklists cl ON ci.checklist_id = cl.id
        JOIN cards ca ON cl.card_id = ca.id
        JOIN columns c ON ca.column_id = c.id
        WHERE ci.id = $1
        "#,
    )
    .bind(item_id)
    .fetch_optional(&state.db)
    .await?;

    match result {
        Some(row) => Ok(row.board_id),
        None => Err(AppError::NotFound("Checklist item not found".to_string())),
    }
}

pub async fn create_checklist(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(card_id): Path<Uuid>,
    Json(req): Json<CreateChecklistRequest>,
) -> AppResult<Json<Checklist>> {
    req.validate()?;

    let board_id = get_card_board_id(&state, card_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let max_position = sqlx::query_as::<_, MaxPositionRow>(
        r#"SELECT MAX(position) as max_pos FROM checklists WHERE card_id = $1"#,
    )
    .bind(card_id)
    .fetch_one(&state.db)
    .await?;

    let position = max_position.max_pos.unwrap_or(0.0) + 65536.0;

    let checklist = sqlx::query_as::<_, Checklist>(
        r#"
        INSERT INTO checklists (card_id, title, position)
        VALUES ($1, $2, $3)
        RETURNING id, card_id, title, position, created_at, updated_at
        "#,
    )
    .bind(card_id)
    .bind(&req.title)
    .bind(position)
    .fetch_one(&state.db)
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
    .bind("checklist_created")
    .bind(EntityType::Card.as_str())
    .bind(card_id)
    .bind(serde_json::json!({ "card_title": card.map(|c| c.title), "checklist_title": checklist.title }))
    .execute(&state.db)
    .await?;

    Ok(Json(checklist))
}

pub async fn update_checklist(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(checklist_id): Path<Uuid>,
    Json(req): Json<UpdateChecklistRequest>,
) -> AppResult<Json<Checklist>> {
    req.validate()?;

    let board_id = get_checklist_board_id(&state, checklist_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let checklist = sqlx::query_as::<_, Checklist>(
        r#"
        UPDATE checklists
        SET title = COALESCE($1, title)
        WHERE id = $2
        RETURNING id, card_id, title, position, created_at, updated_at
        "#,
    )
    .bind(&req.title)
    .bind(checklist_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(checklist))
}

pub async fn delete_checklist(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(checklist_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let board_id = get_checklist_board_id(&state, checklist_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let checklist = sqlx::query_as::<_, ChecklistCardIdTitleRow>(
        r#"SELECT card_id, title FROM checklists WHERE id = $1"#,
    )
    .bind(checklist_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Checklist not found".to_string()))?;

    sqlx::query(r#"DELETE FROM checklists WHERE id = $1"#)
        .bind(checklist_id)
        .execute(&state.db)
        .await?;

    let card = sqlx::query_as::<_, TitleRow>(
        r#"SELECT title FROM cards WHERE id = $1"#,
    )
    .bind(checklist.card_id)
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
    .bind("checklist_deleted")
    .bind(EntityType::Card.as_str())
    .bind(checklist.card_id)
    .bind(serde_json::json!({ "card_title": card.map(|c| c.title), "checklist_title": checklist.title }))
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "message": "Checklist deleted" })))
}

pub async fn create_item(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(checklist_id): Path<Uuid>,
    Json(req): Json<CreateChecklistItemRequest>,
) -> AppResult<Json<ChecklistItem>> {
    req.validate()?;

    let board_id = get_checklist_board_id(&state, checklist_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let max_position = sqlx::query_as::<_, MaxPositionRow>(
        r#"SELECT MAX(position) as max_pos FROM checklist_items WHERE checklist_id = $1"#,
    )
    .bind(checklist_id)
    .fetch_one(&state.db)
    .await?;

    let position = max_position.max_pos.unwrap_or(0.0) + 65536.0;

    let item = sqlx::query_as::<_, ChecklistItem>(
        r#"
        INSERT INTO checklist_items (checklist_id, content, position)
        VALUES ($1, $2, $3)
        RETURNING id, checklist_id, content, is_completed, position, created_at, updated_at
        "#,
    )
    .bind(checklist_id)
    .bind(&req.content)
    .bind(position)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(item))
}

pub async fn update_item(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(item_id): Path<Uuid>,
    Json(req): Json<UpdateChecklistItemRequest>,
) -> AppResult<Json<ChecklistItem>> {
    req.validate()?;

    let board_id = get_checklist_item_board_id(&state, item_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let item = sqlx::query_as::<_, ChecklistItem>(
        r#"
        UPDATE checklist_items
        SET content = COALESCE($1, content)
        WHERE id = $2
        RETURNING id, checklist_id, content, is_completed, position, created_at, updated_at
        "#,
    )
    .bind(&req.content)
    .bind(item_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(item))
}

pub async fn delete_item(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(item_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let board_id = get_checklist_item_board_id(&state, item_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    sqlx::query(r#"DELETE FROM checklist_items WHERE id = $1"#)
        .bind(item_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "message": "Checklist item deleted" })))
}

pub async fn toggle_item(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(item_id): Path<Uuid>,
) -> AppResult<Json<ChecklistItem>> {
    let board_id = get_checklist_item_board_id(&state, item_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let current = sqlx::query_as::<_, ChecklistItemStateRow>(
        r#"SELECT is_completed, checklist_id, content FROM checklist_items WHERE id = $1"#,
    )
    .bind(item_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Checklist item not found".to_string()))?;

    let new_completed = !current.is_completed;

    let item = sqlx::query_as::<_, ChecklistItem>(
        r#"
        UPDATE checklist_items
        SET is_completed = $1
        WHERE id = $2
        RETURNING id, checklist_id, content, is_completed, position, created_at, updated_at
        "#,
    )
    .bind(new_completed)
    .bind(item_id)
    .fetch_one(&state.db)
    .await?;

    let checklist = sqlx::query_as::<_, ChecklistCardIdTitleRow>(
        r#"SELECT card_id, title FROM checklists WHERE id = $1"#,
    )
    .bind(current.checklist_id)
    .fetch_optional(&state.db)
    .await?;

    if let Some(cl) = checklist {
        let card = sqlx::query_as::<_, TitleRow>(
            r#"SELECT title FROM cards WHERE id = $1"#,
        )
        .bind(cl.card_id)
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
        .bind(if new_completed { "checklist_item_completed" } else { "checklist_item_uncompleted" })
        .bind(EntityType::Card.as_str())
        .bind(cl.card_id)
        .bind(serde_json::json!({ 
            "card_title": card.map(|c| c.title), 
            "checklist_title": cl.title,
            "item_content": current.content
        }))
        .execute(&state.db)
        .await?;
    }

    Ok(Json(item))
}
