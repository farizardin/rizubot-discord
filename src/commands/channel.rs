use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Lists all channels in the community
#[poise::command(slash_command, prefix_command, subcommands("fetch"))]
pub async fn channel(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Usage: `/channel fetch`").await?;
    Ok(())
}

/// Fetch channels
#[poise::command(slash_command, prefix_command)]
pub async fn fetch(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    if let Some(guild_id) = ctx.guild_id() {
        let channels = guild_id.channels(ctx.http()).await?;
        
        let mut categories: std::collections::HashMap<Option<serenity::ChannelId>, Vec<serenity::GuildChannel>> = std::collections::HashMap::new();
        
        for (_, channel) in channels {
            if channel.kind == serenity::ChannelType::Category {
                categories.entry(Some(channel.id)).or_insert_with(Vec::new);
            } else {
                categories.entry(channel.parent_id).or_insert_with(Vec::new).push(channel);
            }
        }

        let mut message_content = String::new();

        // Get channels without category first
        if let Some(uncategorized) = categories.get(&None) {
            if !uncategorized.is_empty() {
                message_content.push_str("**Uncategorized**\n");
                for (idx, channel) in uncategorized.iter().enumerate() {
                    message_content.push_str(&format!("{}. **{}** - <#{}>\n", idx + 1, channel.name, channel.id));
                }
                message_content.push_str("\n\n");
            }
        }

        // Get categorized channels
        for (parent_id, channels_in_cat) in &categories {
            if let Some(parent_id) = parent_id {
                let category_name = if let Ok(category_channel) = parent_id.to_channel(ctx.http()).await {
                    category_channel.category().map(|c| c.name).unwrap_or_else(|| "Unknown Category".to_string())
                } else {
                    "Unknown Category".to_string()
                };

                message_content.push_str(&format!("**{}**\n", category_name));
                for (idx, channel) in channels_in_cat.iter().enumerate() {
                    message_content.push_str(&format!("{}. **{}** - <#{}>\n", idx + 1, channel.name, channel.id));
                }
                message_content.push_str("\n\n");
            }
        }

        if message_content.is_empty() {
            ctx.say("No channels found.").await?;
        } else {
            let chunks = crate::utils::split_smart(&message_content);
            for chunk in chunks {
                ctx.say(chunk).await?;
            }
        }

    } else {
        ctx.say("This command can only be used in a server.").await?;
    }

    Ok(())
}
