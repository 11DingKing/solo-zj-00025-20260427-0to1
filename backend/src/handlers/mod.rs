pub mod auth;
pub mod boards;
pub mod board_members;
pub mod columns;
pub mod cards;
pub mod tags;
pub mod checklists;
pub mod activities;

use axum::Json;
use serde_json::json;

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
