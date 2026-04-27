use crate::auth::{generate_token, hash_password, verify_password};
use crate::db::AppState;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::CurrentUser;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, User, UserResponse};
use axum::{extract::State, Json};
use std::sync::Arc;
use validator::Validate;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    req.validate()?;

    let existing = sqlx::query!(
        r#"SELECT id FROM users WHERE username = $1 OR email = $2"#,
        req.username,
        req.email
    )
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict(
            "Username or email already exists".to_string(),
        ));
    }

    let password_hash = hash_password(&req.password)?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, username, email, password_hash, created_at, updated_at
        "#,
    )
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await?;

    let token = generate_token(user.id, &state.config)?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE email = $1"#,
    )
    .bind(&req.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let token = generate_token(user.id, &state.config)?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}

pub async fn get_current_user(
    current_user: CurrentUser,
) -> AppResult<Json<UserResponse>> {
    Ok(Json(UserResponse::from(current_user.user)))
}
