use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Responds with Ping!
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// Shows your user profile
#[poise::command(slash_command, prefix_command)]
pub async fn profile(ctx: Context<'_>) -> Result<(), Error> {
    let discord_id = ctx.author().id.to_string();
    let username = &ctx.author().name;
    
    let user_service = &ctx.data().user_service;
    
    // Register user or get profile
    let user = user_service.get_or_create_user(&discord_id, username).await?;
    
    let response = format!("**Profile for {}**\nLevel: {}\nXP: {}", user.username, user.level, user.xp);
    ctx.say(response).await?;
    
    Ok(())
}

/// Adds some XP (simulation)
#[poise::command(slash_command, prefix_command)]
pub async fn work(ctx: Context<'_>) -> Result<(), Error> {
    let discord_id = ctx.author().id.to_string();
    let username = &ctx.author().name;
    
    let user_service = &ctx.data().user_service;
    
    // Add 10 XP
    let user = user_service.add_xp(&discord_id, username, 10).await?;
    
    let response = format!("You worked hard and gained 10 XP! Total XP: {}", user.xp);
    ctx.say(response).await?;
    
    Ok(())
}
