#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReactionRole {
    pub id: i64,
    pub message_id: String,
    pub channel_id: String,
    pub role_id: String,
    pub role_name: Option<String>,
    pub emoji: String,
}
