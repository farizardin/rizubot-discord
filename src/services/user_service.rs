use crate::models::user::User;
use crate::repositories::user_repository::UserRepository;
use sqlx::Error;

#[derive(Clone)]
pub struct UserService {
    repo: UserRepository,
}

impl UserService {
    pub fn new(repo: UserRepository) -> Self {
        Self { repo }
    }

    pub async fn get_or_create_user(&self, discord_id: &str, username: &str) -> Result<User, Error> {
        // Try to get user first
        if let Some(user) = self.repo.get_by_discord_id(discord_id).await? {
            return Ok(user);
        }

        // If not found, create new user
        self.repo.create_user(discord_id, username).await
    }

    pub async fn add_xp(&self, discord_id: &str, username: &str, xp_to_add: i64) -> Result<User, Error> {
        // Ensure user exists first
        self.get_or_create_user(discord_id, username).await?;
        
        // Update XP
        self.repo.update_xp(discord_id, xp_to_add).await
    }
}
