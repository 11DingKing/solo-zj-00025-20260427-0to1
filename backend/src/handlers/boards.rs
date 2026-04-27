use crate::cache::CacheService;
use crate::db::AppState;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::CurrentUser;
use crate::models::{
    Activity, Board, BoardMember, BoardRole, CreateBoardRequest, EntityType, UpdateBoardRequest,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct ListBoardsQuery {
    pub search: Option<String>,
}

fn boards_cache_key(user_id: Uuid) -> String {
    format!("boards:user:{}", user_id)
}

fn board_cache_key(board_id: Uuid) -> String {
    format!("board:{}", board_id)
}

pub async fn list_boards(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    query: Query<ListBoardsQuery>,
) -> AppResult<Json<Vec<Board>>> {
    let cache = CacheService::new(state.redis.clone());
    let cache_key = boards_cache_key(current_user.user.id);

    if query.search.is_none() {
        if let Some(cached) = cache.get::<Vec<Board>>(&cache_key).await? {
            return Ok(Json(cached));
        }
    }

    let boards = if let Some(search) = &query.search {
        let search_pattern = format!("%{}%", search);
        sqlx::query_as::<_, Board>(
            r#"
            SELECT DISTINCT b.* FROM boards b
            LEFT JOIN board_members bm ON b.id = bm.board_id
            WHERE (b.owner_id = $1 OR bm.user_id = $1)
            AND b.name ILIKE $2
            ORDER BY b.created_at DESC
            "#,
        )
        .bind(current_user.user.id)
        .bind(search_pattern)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, Board>(
            r#"
            SELECT DISTINCT b.* FROM boards b
            LEFT JOIN board_members bm ON b.id = bm.board_id
            WHERE b.owner_id = $1 OR bm.user_id = $1
            ORDER BY b.created_at DESC
            "#,
        )
        .bind(current_user.user.id)
        .fetch_all(&state.db)
        .await?
    };

    if query.search.is_none() {
        cache.set(&cache_key, &boards, 300).await?;
    }

    Ok(Json(boards))
}

pub async fn create_board(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(req): Json<CreateBoardRequest>,
) -> AppResult<Json<Board>> {
    req.validate()?;

    let mut tx = state.db.begin().await?;

    let board = sqlx::query_as::<_, Board>(
        r#"
        INSERT INTO boards (name, description, owner_id)
        VALUES ($1, $2, $3)
        RETURNING id, name, description, owner_id, created_at, updated_at
        "#,
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(current_user.user.id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO board_members (board_id, user_id, role)
        VALUES ($1, $2, $3)
        "#,
        board.id,
        current_user.user.id,
        BoardRole::Owner.as_str()
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        board.id,
        current_user.user.id,
        "created",
        EntityType::Board.as_str(),
        board.id,
        serde_json::json!({ "name": board.name })
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&boards_cache_key(current_user.user.id)).await?;

    Ok(Json(board))
}

pub async fn get_board(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(board_id): Path<Uuid>,
) -> AppResult<Json<Board>> {
    let cache = CacheService::new(state.redis.clone());
    let cache_key = board_cache_key(board_id);

    if let Some(cached) = cache.get::<Board>(&cache_key).await? {
        verify_board_access(&state, current_user.user.id, board_id).await?;
        return Ok(Json(cached));
    }

    verify_board_access(&state, current_user.user.id, board_id).await?;

    let board = sqlx::query_as::<_, Board>(
        r#"SELECT id, name, description, owner_id, created_at, updated_at FROM boards WHERE id = $1"#,
    )
    .bind(board_id)
    .fetch_one(&state.db)
    .await?;

    cache.set(&cache_key, &board, 300).await?;

    Ok(Json(board))
}

pub async fn update_board(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(board_id): Path<Uuid>,
    Json(req): Json<UpdateBoardRequest>,
) -> AppResult<Json<Board>> {
    req.validate()?;

    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_edit_board()).await?;

    let board = sqlx::query_as::<_, Board>(
        r#"
        UPDATE boards
        SET 
            name = COALESCE($1, name),
            description = COALESCE($2, description)
        WHERE id = $3
        RETURNING id, name, description, owner_id, created_at, updated_at
        "#,
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(board_id)
    .fetch_one(&state.db)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        board_id,
        current_user.user.id,
        "updated",
        EntityType::Board.as_str(),
        board_id,
        serde_json::json!({ "name": req.name, "description": req.description })
    )
    .execute(&state.db)
    .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&board_cache_key(board_id)).await?;
    cache.delete(&boards_cache_key(current_user.user.id)).await?;

    Ok(Json(board))
}

pub async fn delete_board(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(board_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    verify_board_permission(&state, current_user.user.id, board_id, |r| matches!(r, BoardRole::Owner)).await?;

    let board = sqlx::query_as::<_, Board>(
        r#"SELECT id, name, description, owner_id, created_at, updated_at FROM boards WHERE id = $1"#,
    )
    .bind(board_id)
    .fetch_one(&state.db)
    .await?;

    sqlx::query!(r#"DELETE FROM boards WHERE id = $1"#, board_id)
        .execute(&state.db)
        .await?;

    let cache = CacheService::new(state.redis.clone());
    cache.delete(&board_cache_key(board_id)).await?;
    cache.delete(&boards_cache_key(current_user.user.id)).await?;
    cache.delete_pattern(&format!("board:{}:*", board_id)).await?;

    Ok(Json(serde_json::json!({ "message": "Board deleted", "name": board.name })))
}

pub async fn verify_board_access(
    state: &Arc<AppState>,
    user_id: Uuid,
    board_id: Uuid,
) -> AppResult<()> {
    let result = sqlx::query!(
        r#"
        SELECT 1 as exists FROM boards b
        LEFT JOIN board_members bm ON b.id = bm.board_id
        WHERE b.id = $1 AND (b.owner_id = $2 OR bm.user_id = $2)
        LIMIT 1
        "#,
        board_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?;

    if result.is_none() {
        return Err(AppError::Forbidden("Access denied to this board".to_string()));
    }

    Ok(())
}

pub async fn get_user_board_role(
    state: &Arc<AppState>,
    user_id: Uuid,
    board_id: Uuid,
) -> AppResult<BoardRole> {
    let board = sqlx::query!(
        r#"SELECT owner_id FROM boards WHERE id = $1"#,
        board_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

    if board.owner_id == user_id {
        return Ok(BoardRole::Owner);
    }

    let member = sqlx::query!(
        r#"SELECT role FROM board_members WHERE board_id = $1 AND user_id = $2"#,
        board_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?;

    match member {
        Some(m) => Ok(BoardRole::from_str(&m.role)),
        None => Err(AppError::Forbidden("Access denied to this board".to_string())),
    }
}

pub async fn verify_board_permission<F>(
    state: &Arc<AppState>,
    user_id: Uuid,
    board_id: Uuid,
    permission_check: F,
) -> AppResult<()>
where
    F: Fn(&BoardRole) -> bool,
{
    let role = get_user_board_role(state, user_id, board_id).await?;
    
    if !permission_check(&role) {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    Ok(())
}
