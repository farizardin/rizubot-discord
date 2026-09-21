use crate::models::reaction_role::ReactionRole;
use crate::repositories::reaction_role_repository::ReactionRoleRepository;
use std::sync::Arc;

pub struct ReactionRoleService {
    repo: Arc<ReactionRoleRepository>,
}

impl ReactionRoleService {
    pub fn new(repo: ReactionRoleRepository) -> Self {
        Self { repo: Arc::new(repo) }
    }

    pub async fn create_reaction_role(
        &self,
        message_id: &str,
        channel_id: &str,
        role_id: &str,
        role_name: Option<&str>,
        emoji: &str,
    ) -> Result<ReactionRole, sqlx::Error> {
        self.repo.create(message_id, channel_id, role_id, role_name, emoji).await
    }

    pub async fn get_by_message_and_emoji(
        &self,
        message_id: &str,
        emoji: &str,
    ) -> Result<Option<ReactionRole>, sqlx::Error> {
        self.repo.find_by_message_and_emoji(message_id, emoji).await
    }

    pub async fn delete_reaction_role(&self, id: i64) -> Result<bool, sqlx::Error> {
        self.repo.delete_by_id(id).await
    }

    pub async fn get_all_reaction_roles(&self) -> Result<Vec<ReactionRole>, sqlx::Error> {
        self.repo.get_all().await
    }
}
