use crate::cache::CacheService;
use crate::db::AppState;
use crate::errors::{AppError, AppResult};
use crate::handlers::boards::{verify_board_access, verify_board_permission};
use crate::middleware::auth::CurrentUser;
use crate::models::{
    BoardMember, BoardMemberWithUser, BoardRole, EntityType, InviteMemberRequest,
    UpdateMemberRoleRequest, UserResponse,
};
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn list_members(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(board_id): Path<Uuid>,
) -> AppResult<Json<Vec<BoardMemberWithUser>>> {
    verify_board_access(&state, current_user.user.id, board_id).await?;

    let members = sqlx::query_as::<_, BoardMemberWithUser>(
        r#"
        SELECT 
            bm.id, bm.board_id, bm.user_id, 
            u.username, u.email,
            bm.role, bm.joined_at
        FROM board_members bm
        JOIN users u ON bm.user_id = u.id
        WHERE bm.board_id = $1
        ORDER BY bm.joined_at
        "#,
    )
    .bind(board_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(members))
}

pub async fn invite_member(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(board_id): Path<Uuid>,
    Json(req): Json<InviteMemberRequest>,
) -> AppResult<Json<BoardMemberWithUser>> {
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_manage_members()).await?;

    let user = sqlx::query!(
        r#"SELECT id, username, email FROM users WHERE email = $1"#,
        req.email
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if user.id == current_user.user.id {
        return Err(AppError::BadRequest("Cannot invite yourself".to_string()));
    }

    let role = BoardRole::from_str(&req.role);
    if matches!(role, BoardRole::Owner) {
        return Err(AppError::BadRequest("Cannot assign owner role".to_string()));
    }

    let existing = sqlx::query!(
        r#"SELECT id FROM board_members WHERE board_id = $1 AND user_id = $2"#,
        board_id,
        user.id
    )
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("User is already a member".to_string()));
    }

    let mut tx = state.db.begin().await?;

    let member = sqlx::query_as::<_, BoardMember>(
        r#"
        INSERT INTO board_members (board_id, user_id, role)
        VALUES ($1, $2, $3)
        RETURNING id, board_id, user_id, role, joined_at
        "#,
    )
    .bind(board_id)
    .bind(user.id)
    .bind(role.as_str())
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        board_id,
        current_user.user.id,
        "invited",
        EntityType::Member.as_str(),
        user.id,
        serde_json::json!({ "username": user.username, "role": role.as_str() })
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(BoardMemberWithUser {
        id: member.id,
        board_id: member.board_id,
        user_id: member.user_id,
        username: user.username,
        email: user.email,
        role: member.role,
        joined_at: member.joined_at,
    }))
}

pub async fn update_member_role(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path((board_id, target_user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> AppResult<Json<BoardMemberWithUser>> {
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_manage_members()).await?;

    if target_user_id == current_user.user.id {
        return Err(AppError::BadRequest("Cannot change your own role".to_string()));
    }

    let new_role = BoardRole::from_str(&req.role);
    if matches!(new_role, BoardRole::Owner) {
        return Err(AppError::BadRequest("Cannot assign owner role".to_string()));
    }

    let member = sqlx::query_as::<_, BoardMember>(
        r#"
        UPDATE board_members
        SET role = $1
        WHERE board_id = $2 AND user_id = $3
        RETURNING id, board_id, user_id, role, joined_at
        "#,
    )
    .bind(new_role.as_str())
    .bind(board_id)
    .bind(target_user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

    let user = sqlx::query!(
        r#"SELECT username, email FROM users WHERE id = $1"#,
        target_user_id
    )
    .fetch_one(&state.db)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        board_id,
        current_user.user.id,
        "role_changed",
        EntityType::Member.as_str(),
        target_user_id,
        serde_json::json!({ "username": user.username, "new_role": new_role.as_str() })
    )
    .execute(&state.db)
    .await?;

    Ok(Json(BoardMemberWithUser {
        id: member.id,
        board_id: member.board_id,
        user_id: member.user_id,
        username: user.username,
        email: user.email,
        role: member.role,
        joined_at: member.joined_at,
    }))
}

pub async fn remove_member(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path((board_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    verify_board_permission(&state, current_user.user.id, board_id, |r| r.can_manage_members()).await?;

    if target_user_id == current_user.user.id {
        return Err(AppError::BadRequest("Cannot remove yourself".to_string()));
    }

    let user = sqlx::query!(
        r#"SELECT username FROM users WHERE id = $1"#,
        target_user_id
    )
    .fetch_optional(&state.db)
    .await?;

    let result = sqlx::query!(
        r#"DELETE FROM board_members WHERE board_id = $1 AND user_id = $2"#,
        board_id,
        target_user_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Member not found".to_string()));
    }

    sqlx::query!(
        r#"
        INSERT INTO activities (board_id, user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        board_id,
        current_user.user.id,
        "removed",
        EntityType::Member.as_str(),
        target_user_id,
        serde_json::json!({ "username": user.map(|u| u.username) })
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "message": "Member removed" })))
}
