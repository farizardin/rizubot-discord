mod models;
mod repositories;
mod services;
mod commands;
pub mod utils;

use anyhow::Context as AnyhowContext;
use poise::serenity_prelude as serenity;
use sqlx::sqlite::{SqlitePoolOptions, SqliteConnectOptions};
use std::str::FromStr;
use songbird::SerenityInit;

use crate::repositories::user_repository::UserRepository;
use crate::services::user_service::UserService;
use crate::repositories::reaction_role_repository::ReactionRoleRepository;
use crate::services::reaction_role_service::ReactionRoleService;

// Custom Error type
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

// App Data that will be passed to commands
pub struct Data {
    pub user_service: UserService,
    pub reaction_role_service: ReactionRoleService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists
    let _ = dotenvy::dotenv();

    // Setup tracing for songbird debug logs
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("songbird=debug".parse().unwrap())
                .add_directive("serenity=warn".parse().unwrap())
        )
        .init();

    // Get Discord token
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:rizubot.db".to_string());

    // Connect to database
    let connect_options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .context("Failed to connect to database")?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    // Initialize Repository and Service
    let user_repo = UserRepository::new(pool.clone());
    let user_service = UserService::new(user_repo);
    let rr_repo = ReactionRoleRepository::new(pool);
    let reaction_role_service = ReactionRoleService::new(rr_repo);

    // Setup Poise Framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::general::ping(),
                commands::general::profile(),
                commands::general::work(),
                commands::ask::ask(),
                commands::waifu::waifu(),
                commands::channel::channel(),
                commands::dev::dev(),
                commands::join::join(),
                commands::reactionrole::reactionrole(),
                commands::play::play(),
                commands::play::queue(),
                commands::play::stop(),
                commands::play::skip(),
                commands::play::clear(),
                commands::play::jump(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(async move {
                    if let poise::serenity_prelude::FullEvent::ReactionAdd { add_reaction } = event {
                        let emoji = add_reaction.emoji.to_string();
                        let msg_id = add_reaction.message_id.to_string();
                        
                        if let Ok(Some(rr)) = data.reaction_role_service.get_by_message_and_emoji(&msg_id, &emoji).await {
                            if let Ok(role_id) = rr.role_id.parse::<u64>() {
                                if let Some(guild_id) = add_reaction.guild_id {
                                    if let Some(user_id) = add_reaction.user_id {
                                        let _ = ctx.http.add_member_role(guild_id, user_id, serenity::RoleId::new(role_id), Some("Reaction Role")).await;
                                    }
                                }
                            }
                        }
                    } else if let poise::serenity_prelude::FullEvent::ReactionRemove { removed_reaction } = event {
                        let emoji = removed_reaction.emoji.to_string();
                        let msg_id = removed_reaction.message_id.to_string();
                        
                        if let Ok(Some(rr)) = data.reaction_role_service.get_by_message_and_emoji(&msg_id, &emoji).await {
                            if let Ok(role_id) = rr.role_id.parse::<u64>() {
                                if let Some(guild_id) = removed_reaction.guild_id {
                                    if let Some(user_id) = removed_reaction.user_id {
                                        let _ = ctx.http.remove_member_role(guild_id, user_id, serenity::RoleId::new(role_id), Some("Reaction Role")).await;
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { user_service, reaction_role_service })
            })
        })
        .build();

    // Setup Serenity Client
    let intents = serenity::GatewayIntents::non_privileged() 
        | serenity::GatewayIntents::MESSAGE_CONTENT 
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS
        | serenity::GatewayIntents::GUILD_VOICE_STATES;
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .await
        .context("Error creating client")?;

    println!("Starting bot...");
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}
