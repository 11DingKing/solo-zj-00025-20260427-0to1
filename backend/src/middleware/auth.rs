use crate::auth::verify_token;
use crate::db::AppState;
use crate::errors::{AppError, AppResult};
use crate::models::User;
use axum::{
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub user: User,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("User not found in request".to_string()))?;

        Ok(CurrentUser { user })
    }
}

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(auth_header) if auth_header.starts_with("Bearer ") => {
            &auth_header[7..]
        }
        _ => {
            return Err(AppError::Unauthorized(
                "Missing or invalid authorization header".to_string(),
            ))
        }
    };

    let claims = verify_token(token, &state.config)?;
    
    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE id = $1"#
    )
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
