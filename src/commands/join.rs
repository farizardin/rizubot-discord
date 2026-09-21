use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Joins the voice channel you are currently in
#[poise::command(slash_command, prefix_command)]
pub async fn join(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("Command ini hanya bisa digunakan di dalam server (Guild).").await?;
            return Ok(());
        }
    };

    let channel_id = ctx.guild().and_then(|g| g.voice_states.get(&ctx.author().id).and_then(|vs| vs.channel_id));

    match channel_id {
        Some(channel) => {
            let manager = songbird::get(ctx.serenity_context())
                .await
                .expect("Songbird Voice client placed in at initialization.")
                .clone();
            
            let _handler = manager.join(guild_id, channel).await;

            println!("[Voice] Joined target channel: {}", channel);
            ctx.say(format!("Berhasil! Bot telah bergabung ke voice channel <#{}>.", channel)).await?;
        }
        None => {
            ctx.say("Anda harus bergabung ke sebuah voice channel terlebih dahulu!").await?;
        }
    }

    Ok(())
}
