use sqlx::SqlitePool;
use crate::models::reaction_role::ReactionRole;

pub struct ReactionRoleRepository {
    pool: SqlitePool,
}

impl ReactionRoleRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        message_id: &str,
        channel_id: &str,
        role_id: &str,
        role_name: Option<&str>,
        emoji: &str,
    ) -> Result<ReactionRole, sqlx::Error> {
        let result = sqlx::query_as::<_, ReactionRole>(
            r#"
            INSERT INTO reaction_roles (message_id, channel_id, role_id, role_name, emoji)
            VALUES (?1, ?2, ?3, ?4, ?5)
            RETURNING id, message_id, channel_id, role_id, role_name, emoji
            "#
        )
        .bind(message_id)
        .bind(channel_id)
        .bind(role_id)
        .bind(role_name)
        .bind(emoji)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn find_by_message_and_emoji(
        &self,
        message_id: &str,
        emoji: &str,
    ) -> Result<Option<ReactionRole>, sqlx::Error> {
        let result = sqlx::query_as::<_, ReactionRole>(
            r#"
            SELECT id, message_id, channel_id, role_id, role_name, emoji
            FROM reaction_roles
            WHERE message_id = ?1 AND emoji = ?2
            "#
        )
        .bind(message_id)
        .bind(emoji)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM reaction_roles
            WHERE id = ?1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_all(&self) -> Result<Vec<ReactionRole>, sqlx::Error> {
        let results = sqlx::query_as::<_, ReactionRole>(
            r#"
            SELECT id, message_id, channel_id, role_id, role_name, emoji
            FROM reaction_roles
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}
