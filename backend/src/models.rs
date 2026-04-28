use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, sqlx::FromRow)]
pub struct IdRow {
    pub id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
pub struct BoardIdRow {
    pub board_id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
pub struct BoardIdNameRow {
    pub board_id: Uuid,
    pub name: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct TitleRow {
    pub title: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct NameRow {
    pub name: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct IdNameBoardIdRow {
    pub id: Uuid,
    pub name: String,
    pub board_id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CardIdTitleRow {
    pub card_id: Uuid,
    pub title: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ChecklistCardIdTitleRow {
    pub card_id: Uuid,
    pub title: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ChecklistItemStateRow {
    pub is_completed: bool,
    pub checklist_id: Uuid,
    pub content: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct MaxPositionRow {
    pub max_pos: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ExistsRow {
    pub exists: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct OwnerIdRow {
    pub owner_id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RoleRow {
    pub role: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UsernameEmailRow {
    pub username: String,
    pub email: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserIdUsernameEmailRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UsernameRow {
    pub username: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PositionRow {
    pub position: f64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct BoardIdPositionRow {
    pub board_id: Uuid,
    pub position: f64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ColumnIdTitleRow {
    pub column_id: Uuid,
    pub title: String,
}

// 用户相关
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
        }
    }
}

// 看板相关
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBoardRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateBoardRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

// 看板成员
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BoardRole {
    Owner,
    Admin,
    Member,
}

impl BoardRole {
    pub fn as_str(&self) -> &str {
        match self {
            BoardRole::Owner => "owner",
            BoardRole::Admin => "admin",
            BoardRole::Member => "member",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "owner" => BoardRole::Owner,
            "admin" => BoardRole::Admin,
            _ => BoardRole::Member,
        }
    }

    pub fn can_edit_board(&self) -> bool {
        matches!(self, BoardRole::Owner | BoardRole::Admin)
    }

    pub fn can_manage_members(&self) -> bool {
        matches!(self, BoardRole::Owner | BoardRole::Admin)
    }

    pub fn can_edit_cards(&self) -> bool {
        true
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BoardMember {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BoardMemberWithUser {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

// 列表相关
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Column {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub position: f64,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateColumnRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    pub after_column_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateColumnRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderColumnRequest {
    pub after_column_id: Option<Uuid>,
}

// 卡片相关
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
}

impl Priority {
    pub fn as_str(&self) -> &str {
        match self {
            Priority::P0 => "P0",
            Priority::P1 => "P1",
            Priority::P2 => "P2",
            Priority::P3 => "P3",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "P0" => Priority::P0,
            "P1" => Priority::P1,
            "P2" => Priority::P2,
            _ => Priority::P3,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Card {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub position: f64,
    pub priority: String,
    pub due_date: Option<DateTime<Utc>>,
    pub assignee_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCardRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub description: Option<String>,
    pub after_card_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCardRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub assignee_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct MoveCardRequest {
    pub target_column_id: Uuid,
    pub after_card_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CardWithDetails {
    #[serde(flatten)]
    pub card: Card,
    pub assignee: Option<UserResponse>,
    pub tags: Vec<Tag>,
    pub checklists: Vec<ChecklistWithItems>,
}

// 标签相关
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Tag {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTagRequest {
    #[validate(length(min = 1, max = 30))]
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTagRequest {
    #[validate(length(min = 1, max = 30))]
    pub name: Option<String>,
    pub color: Option<String>,
}

// 检查清单相关
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Checklist {
    pub id: Uuid,
    pub card_id: Uuid,
    pub title: String,
    pub position: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub checklist_id: Uuid,
    pub content: String,
    pub is_completed: bool,
    pub position: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ChecklistWithItems {
    #[serde(flatten)]
    pub checklist: Checklist,
    pub items: Vec<ChecklistItem>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChecklistRequest {
    #[validate(length(min = 1, max = 100))]
    pub title: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateChecklistRequest {
    #[validate(length(min = 1, max = 100))]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChecklistItemRequest {
    #[validate(length(min = 1, max = 500))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateChecklistItemRequest {
    #[validate(length(min = 1, max = 500))]
    pub content: Option<String>,
}

// 活动日志相关
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Board,
    Column,
    Card,
    Tag,
    Member,
}

impl EntityType {
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::Board => "board",
            EntityType::Column => "column",
            EntityType::Card => "card",
            EntityType::Tag => "tag",
            EntityType::Member => "member",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Activity {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ActivityWithUser {
    #[serde(flatten)]
    pub activity: Activity,
    pub user: UserResponse,
}

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
}
