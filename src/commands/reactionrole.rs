use crate::{Context, Error};

/// Setup a reaction role message
#[poise::command(slash_command, prefix_command, subcommands("add", "remove", "fetch"))]
pub async fn reactionrole(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Usage: `/reactionrole <add|remove|fetch> ...`").await?;
    Ok(())
}

/// Add a reaction role
#[poise::command(slash_command, prefix_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Role ID"] role_id: String,
    #[description = "Emoji shortcode or unicode"] emoji: String,
    #[description = "Message content"] message_content: String,
) -> Result<(), Error> {
    // Basic permission check (requires Manage Roles)
    if let Some(member) = ctx.author_member().await {
        if let Ok(permissions) = member.permissions(ctx.cache()) {
            if !permissions.manage_roles() && !permissions.administrator() {
                ctx.say("❌ You do not have permission to manage reaction roles.").await?;
                return Ok(());
            }
        }
    }

    let service = &ctx.data().reaction_role_service;
    
    // Create the message in the current channel
    let message = ctx.channel_id().say(ctx.http(), message_content).await?;
    
    // Add the reaction to the message
    let reaction_type = poise::serenity_prelude::ReactionType::Unicode(emoji.clone()); // Assuming emoji is unicode for simplicity. If it's a custom emoji, needs parsing.
    
    if let Err(e) = message.react(ctx.http(), reaction_type).await {
        ctx.say(format!("Failed to add reaction: {}", e)).await?;
        return Ok(());
    }

    // Save to database
    match service.create_reaction_role(&message.id.to_string(), &ctx.channel_id().to_string(), &role_id, None, &emoji).await {
        Ok(_) => {
            ctx.say("Success added reaction role!").await?;
        }
        Err(e) => {
            ctx.say(format!("Failed to save reaction role to database: {}", e)).await?;
        }
    }

    Ok(())
}

/// Remove a reaction role by ID
#[poise::command(slash_command, prefix_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Reaction Role Database ID"] id: i64,
) -> Result<(), Error> {
    if let Some(member) = ctx.author_member().await {
        if let Ok(permissions) = member.permissions(ctx.cache()) {
            if !permissions.manage_roles() && !permissions.administrator() {
                ctx.say("❌ You do not have permission to manage reaction roles.").await?;
                return Ok(());
            }
        }
    }

    let service = &ctx.data().reaction_role_service;

    match service.delete_reaction_role(id).await {
        Ok(true) => {
            ctx.say(format!("Removed reaction role for ID {}", id)).await?;
        }
        Ok(false) => {
            ctx.say(format!("Could not find reaction role with ID {}", id)).await?;
        }
        Err(e) => {
            ctx.say(format!("Error deleting reaction role: {}", e)).await?;
        }
    }

    Ok(())
}

/// List all reaction roles
#[poise::command(slash_command, prefix_command)]
pub async fn fetch(ctx: Context<'_>) -> Result<(), Error> {
    let service = &ctx.data().reaction_role_service;

    match service.get_all_reaction_roles().await {
        Ok(roles) => {
            if roles.is_empty() {
                ctx.say("No reaction roles found.").await?;
            } else {
                let mut message_content = String::from("**Reaction Roles:**\n");
                for rr in roles {
                    let role_name = rr.role_name.unwrap_or_else(|| "Unknown".to_string());
                    message_content.push_str(&format!("**{}** - {} (Role ID: {}, ID: {})\n", rr.emoji, role_name, rr.role_id, rr.id));
                }
                ctx.say(message_content).await?;
            }
        }
        Err(e) => {
            ctx.say(format!("Error fetching reaction roles: {}", e)).await?;
        }
    }

    Ok(())
}
