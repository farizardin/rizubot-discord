use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub discord_id: String,
    pub username: String,
    pub xp: i64,
    pub level: i64,
}
