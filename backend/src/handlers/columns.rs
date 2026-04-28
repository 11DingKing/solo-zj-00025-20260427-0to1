use crate::cache::CacheService;
use crate::db::AppState;
use crate::errors::{AppError, AppResult};
use crate::handlers::boards::{verify_board_access, verify_board_permission};
use crate::middleware::auth::CurrentUser;
use crate::models::{
    BoardIdNameRow, BoardIdPositionRow, BoardIdRow, Column, CreateColumnRequest, EntityType,
    MaxPositionRow, PositionRow, ReorderColumnRequest, UpdateColumnRequest,
};
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

fn columns_cache_key(board_id: Uuid) -> String {
    format!("board:{}:columns", board_id)
}

pub async fn list_columns(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(board_id): Path<Uuid>,
) -> AppResult<Json<Vec<Column>>> {
    verify_board_access(&state, current_user.user.id, board_id).await?;

    let cache = CacheService::new(state.redis.clone());
    let cache_key = columns_cache_key(board_id);

    if let Some(cached) = cache.get::<Vec<Column>>(&cache_key).await? {
        return Ok(Json(cached));
    }

    let columns = sqlx::query_as::<_, Column>(
        r#"
        SELECT id, board_id, name, position, color, created_at, updated_at 
        FROM columns 
        WHERE board_id = $1 
        ORDER BY position
        "#,
    )
    .bind(board_id)
    .fetch_all(&state.db)
    .await?;

    cache.set(&cache_key, &columns, 300).await?;

    Ok(Json(columns))
}

pub async fn create_column(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(board_id): Path<Uuid>,
    Json(req): Json<CreateColumnRequest>,
) -> AppResult<Json<Column>> {
    req.validate()?;
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_board()).await?;

    let position = if let Some(after_column_id) = req.after_column_id {
        let after_column = sqlx::query_as::<_, PositionRow>(
            r#"SELECT position FROM columns WHERE id = $1 AND board_id = $2"#,
        )
        .bind(after_column_id)
        .bind(board_id)
        .fetch_optional(&state.db)
        .await?;

        match after_column {
            Some(col) => col.position + 65536.0,
            None => 65536.0,
        }
    } else {
        let max_position = sqlx::query_as::<_, MaxPositionRow>(
            r#"SELECT MAX(position) as max_pos FROM columns WHERE board_id = $1"#,
        )
        .bind(board_id)
        .fetch_one(&state.db)
        .await?;

        max_position.max_pos.unwrap_or(0.0) + 65536.0
    };

    let column = sqlx::query_as::<_, Column>(
        r#"
        INSERT INTO columns (board_id, name, position)
        VALUES ($1, $2, $3)
        RETURNING id, board_id, name, position, color, created_at, updated_at
        "#,
    )
    .bind(board_id)
    .bind(&req.name)
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
    .bind(EntityType::Column.as_str())
    .bind(column.id)
    .bind(serde_json::json!({ "name": column.name }))
    .execute(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&columns_cache_key(board_id)).await?;

    Ok(Json(column))
}

pub async fn update_column(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(column_id): Path<Uuid>,
    Json(req): Json<UpdateColumnRequest>,
) -> AppResult<Json<Column>> {
    req.validate()?;

    let column = sqlx::query_as::<_, BoardIdRow>(
        r#"SELECT board_id FROM columns WHERE id = $1"#,
    )
    .bind(column_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    verify_board_permission(&state, current_user.user.id, column.board_id, |r| r.can_edit_board()).await?;

    let column = sqlx::query_as::<_, Column>(
        r#"
        UPDATE columns
        SET 
            name = COALESCE($1, name),
            color = COALESCE($2, color)
        WHERE id = $3
        RETURNING id, board_id, name, position, color, created_at, updated_at
        "#,
    )
    .bind(&req.name)
    .bind(&req.color)
    .bind(column_id)
    .fetch_one(&state.db)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(column.board_id)
    .bind(current_user.user.id)
    .bind("updated")
    .bind(EntityType::Column.as_str())
    .bind(column.id)
    .bind(serde_json::json!({ "name": req.name, "color": req.color }))
    .execute(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&columns_cache_key(column.board_id)).await?;

    Ok(Json(column))
}

pub async fn delete_column(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(column_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let column = sqlx::query_as::<_, BoardIdNameRow>(
        r#"SELECT board_id, name FROM columns WHERE id = $1"#,
    )
    .bind(column_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    verify_board_permission(&state, current_user.user.id, column.board_id, |r| r.can_edit_board()).await?;

    sqlx::query(r#"DELETE FROM columns WHERE id = $1"#)
        .bind(column_id)
        .execute(&state.db)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(column.board_id)
    .bind(current_user.user.id)
    .bind("deleted")
    .bind(EntityType::Column.as_str())
    .bind(column_id)
    .bind(serde_json::json!({ "name": column.name }))
    .execute(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&columns_cache_key(column.board_id)).await?;

    Ok(Json(serde_json::json!({ "message": "Column deleted" })))
}

pub async fn reorder_column(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(column_id): Path<Uuid>,
    Json(req): Json<ReorderColumnRequest>,
) -> AppResult<Json<Column>> {
    let column = sqlx::query_as::<_, BoardIdPositionRow>(
        r#"SELECT board_id, position FROM columns WHERE id = $1"#,
    )
    .bind(column_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    verify_board_permission(&state, current_user.user.id, column.board_id, |r| r.can_edit_board()).await?;

    let new_position = if let Some(after_column_id) = req.after_column_id {
        if after_column_id == column_id {
            return Err(AppError::BadRequest("Cannot reorder after itself".to_string()));
        }

        let after_column = sqlx::query_as::<_, PositionRow>(
            r#"SELECT position FROM columns WHERE id = $1 AND board_id = $2"#,
        )
        .bind(after_column_id)
        .bind(column.board_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Reference column not found".to_string()))?;

        let next_column = sqlx::query_as::<_, PositionRow>(
            r#"
            SELECT position FROM columns 
            WHERE board_id = $1 AND position > $2 AND id != $3
            ORDER BY position ASC
            LIMIT 1
            "#,
        )
        .bind(column.board_id)
        .bind(after_column.position)
        .bind(column_id)
        .fetch_optional(&state.db)
        .await?;

        match next_column {
            Some(next) => (after_column.position + next.position) / 2.0,
            None => after_column.position + 65536.0,
        }
    } else {
        let first_column = sqlx::query_as::<_, PositionRow>(
            r#"
            SELECT position FROM columns 
            WHERE board_id = $1 AND id != $2
            ORDER BY position ASC
            LIMIT 1
            "#,
        )
        .bind(column.board_id)
        .bind(column_id)
        .fetch_optional(&state.db)
        .await?;

        match first_column {
            Some(first) => first.position / 2.0,
            None => column.position,
        }
    };

    let column = sqlx::query_as::<_, Column>(
        r#"
        UPDATE columns
        SET position = $1
        WHERE id = $2
        RETURNING id, board_id, name, position, color, created_at, updated_at
        "#,
    )
    .bind(new_position)
    .bind(column_id)
    .fetch_one(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&columns_cache_key(column.board_id)).await?;

    Ok(Json(column))
}
