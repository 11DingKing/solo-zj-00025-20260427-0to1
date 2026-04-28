use crate::cache::CacheService;
use crate::db::AppState;
use crate::errors::{AppError, AppResult};
use crate::handlers::boards::{verify_board_access, verify_board_permission};
use crate::middleware::auth::CurrentUser;
use crate::models::{
    BoardIdRow, Card, CardWithDetails, Checklist, ChecklistItem, ChecklistWithItems,
    ColumnIdTitleRow, CreateCardRequest, EntityType, MaxPositionRow, MoveCardRequest, NameRow,
    Priority, Tag, UpdateCardRequest, UserIdUsernameEmailRow, UserResponse,
};
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

fn cards_cache_key(column_id: Uuid) -> String {
    format!("column:{}:cards", column_id)
}

pub async fn list_cards(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(column_id): Path<Uuid>,
) -> AppResult<Json<Vec<Card>>> {
    let board_id = get_column_board_id(&state, column_id).await?;
    verify_board_access(&state, current_user.user.id, board_id).await?;

    let cache = CacheService::new(state.redis.clone());
    let cache_key = cards_cache_key(column_id);

    if let Some(cached) = cache.get::<Vec<Card>>(&cache_key).await? {
        return Ok(Json(cached));
    }

    let cards = sqlx::query_as::<_, Card>(
        r#"
        SELECT id, column_id, title, description, position, priority, due_date, assignee_id, created_at, updated_at 
        FROM cards 
        WHERE column_id = $1 
        ORDER BY position
        "#,
    )
    .bind(column_id)
    .fetch_all(&state.db)
    .await?;

    cache.set(&cache_key, &cards, 300).await?;

    Ok(Json(cards))
}

pub async fn get_column_board_id(state: &Arc<AppState>, column_id: Uuid) -> AppResult<Uuid> {
    let result = sqlx::query_as::<_, BoardIdRow>(
        r#"SELECT board_id FROM columns WHERE id = $1"#,
    )
    .bind(column_id)
    .fetch_optional(&state.db)
    .await?;

    match result {
        Some(row) => Ok(row.board_id),
        None => Err(AppError::NotFound("Column not found".to_string())),
    }
}

pub async fn get_card_board_id(state: &Arc<AppState>, card_id: Uuid) -> AppResult<Uuid> {
    let result = sqlx::query_as::<_, BoardIdRow>(
        r#"
        SELECT c.board_id 
        FROM cards ca
        JOIN columns c ON ca.column_id = c.id
        WHERE ca.id = $1
        "#,
    )
    .bind(card_id)
    .fetch_optional(&state.db)
    .await?;

    match result {
        Some(row) => Ok(row.board_id),
        None => Err(AppError::NotFound("Card not found".to_string())),
    }
}

pub async fn create_card(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(column_id): Path<Uuid>,
    Json(req): Json<CreateCardRequest>,
) -> AppResult<Json<Card>> {
    req.validate()?;

    let board_id = get_column_board_id(&state, column_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let position = if let Some(after_card_id) = req.after_card_id {
        let after_card = sqlx::query_as::<_, PositionRow>(
            r#"SELECT position FROM cards WHERE id = $1 AND column_id = $2"#,
        )
        .bind(after_card_id)
        .bind(column_id)
        .fetch_optional(&state.db)
        .await?;

        match after_card {
            Some(card) => card.position + 65536.0,
            None => 65536.0,
        }
    } else {
        let max_position = sqlx::query_as::<_, MaxPositionRow>(
            r#"SELECT MAX(position) as max_pos FROM cards WHERE column_id = $1"#,
        )
        .bind(column_id)
        .fetch_one(&state.db)
        .await?;

        max_position.max_pos.unwrap_or(0.0) + 65536.0
    };

    let card = sqlx::query_as::<_, Card>(
        r#"
        INSERT INTO cards (column_id, title, description, position)
        VALUES ($1, $2, $3, $4)
        RETURNING id, column_id, title, description, position, priority, due_date, assignee_id, created_at, updated_at
        "#,
    )
    .bind(column_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(position)
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
    .bind(EntityType::Card.as_str())
    .bind(card.id)
    .bind(serde_json::json!({ "title": card.title }))
    .execute(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&cards_cache_key(column_id)).await?;

    Ok(Json(card))
}

pub async fn get_card(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(card_id): Path<Uuid>,
) -> AppResult<Json<CardWithDetails>> {
    let board_id = get_card_board_id(&state, card_id).await?;
    verify_board_access(&state, current_user.user.id, board_id).await?;

    let card = sqlx::query_as::<_, Card>(
        r#"
        SELECT id, column_id, title, description, position, priority, due_date, assignee_id, created_at, updated_at 
        FROM cards 
        WHERE id = $1
        "#,
    )
    .bind(card_id)
    .fetch_one(&state.db)
    .await?;

    let assignee = if let Some(assignee_id) = card.assignee_id {
        sqlx::query_as::<_, UserIdUsernameEmailRow>(
            r#"SELECT id, username, email FROM users WHERE id = $1"#,
        )
        .bind(assignee_id)
        .fetch_optional(&state.db)
        .await?
        .map(|u| UserResponse {
            id: u.id,
            username: u.username,
            email: u.email,
        })
    } else {
        None
    };

    let tags = sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.id, t.board_id, t.name, t.color, t.created_at
        FROM tags t
        JOIN card_tags ct ON t.id = ct.tag_id
        WHERE ct.card_id = $1
        "#,
    )
    .bind(card_id)
    .fetch_all(&state.db)
    .await?;

    let checklists = sqlx::query_as::<_, Checklist>(
        r#"
        SELECT id, card_id, title, position, created_at, updated_at
        FROM checklists
        WHERE card_id = $1
        ORDER BY position
        "#,
    )
    .bind(card_id)
    .fetch_all(&state.db)
    .await?;

    let mut checklists_with_items = Vec::new();
    for checklist in checklists {
        let items = sqlx::query_as::<_, ChecklistItem>(
            r#"
            SELECT id, checklist_id, content, is_completed, position, created_at, updated_at
            FROM checklist_items
            WHERE checklist_id = $1
            ORDER BY position
            "#,
        )
        .bind(checklist.id)
        .fetch_all(&state.db)
        .await?;

        checklists_with_items.push(ChecklistWithItems { checklist, items });
    }

    Ok(Json(CardWithDetails {
        card,
        assignee,
        tags,
        checklists: checklists_with_items,
    }))
}

pub async fn update_card(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(card_id): Path<Uuid>,
    Json(req): Json<UpdateCardRequest>,
) -> AppResult<Json<Card>> {
    req.validate()?;

    let board_id = get_card_board_id(&state, card_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let old_card = sqlx::query_as::<_, Card>(
        r#"
        SELECT id, column_id, title, description, position, priority, due_date, assignee_id, created_at, updated_at 
        FROM cards 
        WHERE id = $1
        "#,
    )
    .bind(card_id)
    .fetch_one(&state.db)
    .await?;

    let priority = req.priority.as_ref().map(|p| Priority::from_str(p).as_str());

    let card = sqlx::query_as::<_, Card>(
        r#"
        UPDATE cards
        SET 
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            priority = COALESCE($3, priority),
            due_date = $4,
            assignee_id = $5
        WHERE id = $6
        RETURNING id, column_id, title, description, position, priority, due_date, assignee_id, created_at, updated_at
        "#,
    )
    .bind(&req.title)
    .bind(&req.description)
    .bind(priority)
    .bind(req.due_date)
    .bind(req.assignee_id)
    .bind(card_id)
    .fetch_one(&state.db)
    .await?;

    let mut changes = serde_json::Map::new();
    if req.title.is_some() && req.title != Some(old_card.title.clone()) {
        changes.insert("title".to_string(), serde_json::json!({
            "old": old_card.title,
            "new": card.title
        }));
    }
    if req.description.is_some() && req.description != old_card.description {
        changes.insert("description".to_string(), serde_json::json!({
            "changed": true
        }));
    }
    if priority.is_some() && req.priority != Some(old_card.priority.clone()) {
        changes.insert("priority".to_string(), serde_json::json!({
            "old": old_card.priority,
            "new": card.priority
        }));
    }

    if !changes.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(board_id)
        .bind(current_user.user.id)
        .bind("updated")
        .bind(EntityType::Card.as_str())
        .bind(card.id)
        .bind(serde_json::Value::Object(changes))
        .execute(&state.db)
        .await?;
    }

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&cards_cache_key(card.column_id)).await?;

    Ok(Json(card))
}

pub async fn delete_card(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(card_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let board_id = get_card_board_id(&state, card_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let card = sqlx::query_as::<_, ColumnIdTitleRow>(
        r#"SELECT column_id, title FROM cards WHERE id = $1"#,
    )
    .bind(card_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Card not found".to_string()))?;

    sqlx::query(r#"DELETE FROM cards WHERE id = $1"#)
        .bind(card_id)
        .execute(&state.db)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(board_id)
    .bind(current_user.user.id)
    .bind("deleted")
    .bind(EntityType::Card.as_str())
    .bind(card_id)
    .bind(serde_json::json!({ "title": card.title }))
    .execute(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&cards_cache_key(card.column_id)).await?;

    Ok(Json(serde_json::json!({ "message": "Card deleted" })))
}

pub async fn move_card(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(card_id): Path<Uuid>,
    Json(req): Json<MoveCardRequest>,
) -> AppResult<Json<Card>> {
    let board_id = get_card_board_id(&state, card_id).await?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_cards()).await?;

    let target_board_id = get_column_board_id(&state, req.target_column_id).await?;
    if target_board_id != board_id {
        return Err(AppError::BadRequest("Cannot move card to another board".to_string()));
    }

    let old_card = sqlx::query_as::<_, Card>(
        r#"
        SELECT id, column_id, title, description, position, priority, due_date, assignee_id, created_at, updated_at 
        FROM cards 
        WHERE id = $1
        "#,
    )
    .bind(card_id)
    .fetch_one(&state.db)
    .await?;

    let position = if let Some(after_card_id) = req.after_card_id {
        let after_card = sqlx::query_as::<_, PositionRow>(
            r#"SELECT position FROM cards WHERE id = $1 AND column_id = $2"#,
        )
        .bind(after_card_id)
        .bind(req.target_column_id)
        .fetch_optional(&state.db)
        .await?;

        match after_card {
            Some(card) => card.position + 65536.0,
            None => 65536.0,
        }
    } else {
        let max_position = sqlx::query_as::<_, MaxPositionRow>(
            r#"SELECT MAX(position) as max_pos FROM cards WHERE column_id = $1"#,
        )
        .bind(req.target_column_id)
        .fetch_one(&state.db)
        .await?;

        max_position.max_pos.unwrap_or(0.0) + 65536.0
    };

    let card = sqlx::query_as::<_, Card>(
        r#"
        UPDATE cards
        SET column_id = $1, position = $2
        WHERE id = $3
        RETURNING id, column_id, title, description, position, priority, due_date, assignee_id, created_at, updated_at
        "#,
    )
    .bind(req.target_column_id)
    .bind(position)
    .bind(card_id)
    .fetch_one(&state.db)
    .await?;

    let mut activity_details = serde_json::Map::new();
    activity_details.insert("title".to_string(), serde_json::json!(card.title));

    if old_card.column_id != req.target_column_id {
        let old_column = sqlx::query_as::<_, NameRow>(
            r#"SELECT name FROM columns WHERE id = $1"#,
        )
        .bind(old_card.column_id)
        .fetch_optional(&state.db)
        .await?;
        
        let new_column = sqlx::query_as::<_, NameRow>(
            r#"SELECT name FROM columns WHERE id = $1"#,
        )
        .bind(req.target_column_id)
        .fetch_optional(&state.db)
        .await?;

        activity_details.insert(
            "from_column".to_string(),
            serde_json::json!(old_column.map(|c| c.name)),
        );
        activity_details.insert(
            "to_column".to_string(),
            serde_json::json!(new_column.map(|c| c.name)),
        );
    }

    sqlx::query(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(board_id)
    .bind(current_user.user.id)
    .bind("moved")
    .bind(EntityType::Card.as_str())
    .bind(card.id)
    .bind(serde_json::Value::Object(activity_details))
    .execute(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&cards_cache_key(old_card.column_id)).await?;
    cache.delete(&cards_cache_key(req.target_column_id)).await?;

    Ok(Json(card))
}
