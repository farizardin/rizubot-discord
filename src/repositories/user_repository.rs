use crate::models::user::User;
use sqlx::{SqlitePool, Error};

#[derive(Clone)]
pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_by_discord_id(&self, discord_id: &str) -> Result<Option<User>, Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE discord_id = ?")
            .bind(discord_id)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(user)
    }

    pub async fn create_user(&self, discord_id: &str, username: &str) -> Result<User, Error> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (discord_id, username, xp, level) VALUES (?, ?, 0, 1) RETURNING *"
        )
        .bind(discord_id)
        .bind(username)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }

    pub async fn update_xp(&self, discord_id: &str, xp_to_add: i64) -> Result<User, Error> {
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET xp = xp + ? WHERE discord_id = ? RETURNING *"
        )
        .bind(xp_to_add)
        .bind(discord_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
}
