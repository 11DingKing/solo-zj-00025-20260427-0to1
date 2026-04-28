use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

mod config;
mod db;
mod cache;
mod auth;
mod errors;
mod models;
mod handlers;
mod middleware;

use config::Config;
use db::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载配置
    let config = Config::from_env();

    // 连接数据库
    let db_pool = db::create_pool(&config.database_url).await?;
    
    // 运行数据库迁移
    db::run_migrations(&db_pool).await?;

    // 连接 Redis
    let redis_client = cache::create_client(&config.redis_url)?;

    // 创建应用状态
    let state = Arc::new(AppState {
        db: db_pool,
        redis: redis_client,
        config: config.clone(),
    });

    // CORS 配置
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建路由
    let app = Router::new()
        // 公开路由
        .route("/api/health", get(handlers::health_check))
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login))
        // 受保护路由
        .nest("/api", protected_routes(state.clone()))
        .layer(cors)
        .with_state(state);

    // 启动服务器
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn protected_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        // 用户相关
        .route("/auth/me", get(handlers::auth::get_current_user))
        // 看板相关
        .route("/boards", get(handlers::boards::list_boards))
        .route("/boards", post(handlers::boards::create_board))
        .route("/boards/:id", get(handlers::boards::get_board))
        .route("/boards/:id", put(handlers::boards::update_board))
        .route("/boards/:id", delete(handlers::boards::delete_board))
        // 看板成员
        .route("/boards/:id/members", get(handlers::board_members::list_members))
        .route("/boards/:id/members", post(handlers::board_members::invite_member))
        .route("/boards/:id/members/:user_id", put(handlers::board_members::update_member_role))
        .route("/boards/:id/members/:user_id", delete(handlers::board_members::remove_member))
        // 列表相关
        .route("/boards/:board_id/columns", get(handlers::columns::list_columns))
        .route("/boards/:board_id/columns", post(handlers::columns::create_column))
        .route("/columns/:id", put(handlers::columns::update_column))
        .route("/columns/:id", delete(handlers::columns::delete_column))
        .route("/columns/:id/reorder", put(handlers::columns::reorder_column))
        // 卡片相关
        .route("/columns/:column_id/cards", get(handlers::cards::list_cards))
        .route("/columns/:column_id/cards", post(handlers::cards::create_card))
        .route("/cards/:id", get(handlers::cards::get_card))
        .route("/cards/:id", put(handlers::cards::update_card))
        .route("/cards/:id", delete(handlers::cards::delete_card))
        .route("/cards/:id/move", put(handlers::cards::move_card))
        // 标签相关
        .route("/boards/:board_id/tags", get(handlers::tags::list_tags))
        .route("/boards/:board_id/tags", post(handlers::tags::create_tag))
        .route("/tags/:id", put(handlers::tags::update_tag))
        .route("/tags/:id", delete(handlers::tags::delete_tag))
        .route("/cards/:card_id/tags/:tag_id", post(handlers::tags::add_tag_to_card))
        .route("/cards/:card_id/tags/:tag_id", delete(handlers::tags::remove_tag_from_card))
        // 检查清单相关
        .route("/cards/:card_id/checklists", post(handlers::checklists::create_checklist))
        .route("/checklists/:id", put(handlers::checklists::update_checklist))
        .route("/checklists/:id", delete(handlers::checklists::delete_checklist))
        .route("/checklists/:checklist_id/items", post(handlers::checklists::create_item))
        .route("/checklist_items/:id", put(handlers::checklists::update_item))
        .route("/checklist_items/:id", delete(handlers::checklists::delete_item))
        .route("/checklist_items/:id/toggle", put(handlers::checklists::toggle_item))
        // 活动日志
        .route("/boards/:board_id/activities", get(handlers::activities::list_activities))
        // 添加 JWT 认证中间件
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            middleware::auth::require_auth,
        ))
}
