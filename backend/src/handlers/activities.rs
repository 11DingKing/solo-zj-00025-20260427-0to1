use crate::db::AppState;
use crate::errors::{AppError, AppResult};
use crate::handlers::boards::verify_board_access;
use crate::middleware::auth::CurrentUser;
use crate::models::{Activity, ActivityWithUser, UserResponse};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListActivitiesQuery {
    pub limit: Option<i64>,
    pub before: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn list_activities(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(board_id): Path<Uuid>,
    query: Query<ListActivitiesQuery>,
) -> AppResult<Json<Vec<ActivityWithUser>>> {
    verify_board_access(&state, current_user.user.id, board_id).await?;

    let limit = query.limit.unwrap_or(50).min(200);

    let activities: Vec<Activity> = if let Some(before) = query.before {
        sqlx::query_as::<_, Activity>(
            r#"
            SELECT id, board_id, user_id, action, entity_type, entity_id, details, created_at
            FROM activities
            WHERE board_id = $1 AND created_at < $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(board_id)
        .bind(before)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, Activity>(
            r#"
            SELECT id, board_id, user_id, action, entity_type, entity_id, details, created_at
            FROM activities
            WHERE board_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(board_id)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    let user_ids: Vec<Uuid> = activities.iter().map(|a| a.user_id).collect();
    
    let users: Vec<UserResponse> = if user_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query_as::<_, UserResponse>(
            r#"
            SELECT id, username, email
            FROM users
            WHERE id = ANY($1)
            "#,
        )
        .bind(&user_ids)
        .fetch_all(&state.db)
        .await?
    };

    let user_map: std::collections::HashMap<Uuid, UserResponse> = users
        .into_iter()
        .map(|u| (u.id, u))
        .collect();

    let result = activities
        .into_iter()
        .map(|activity| {
            let user = user_map.get(&activity.user_id).cloned().unwrap_or(UserResponse {
                id: activity.user_id,
                username: "Unknown".to_string(),
                email: "".to_string(),
            });
            
            ActivityWithUser { activity, user }
        })
        .collect();

    Ok(Json(result))
}
