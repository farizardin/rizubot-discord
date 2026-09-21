use crate::{Context, Error};
use songbird::input::{YoutubeDl, Compose};

/// Plays a song or audio from a given URL (e.g. YouTube)
#[poise::command(slash_command, prefix_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "URL to play (YouTube, direct MP3, etc.)"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("Command ini hanya bisa digunakan di dalam server (Guild).").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let http_client = reqwest::Client::new();
        let mut src = YoutubeDl::new(http_client, url.clone());

        match src.aux_metadata().await {
            Ok(metadata) => {
                let title = metadata.title.unwrap_or_else(|| url.clone());
                println!("[Music] Resolved: {} (duration: {:?})", title, metadata.duration);

                let mut ytdl = std::process::Command::new("yt-dlp")
                    .args(&["-f", "ba", "-q", "-o", "-", &url])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .expect("Failed to spawn yt-dlp");

                let stdout = ytdl.stdout.take().unwrap();
                let ffmpeg = std::process::Command::new("ffmpeg")
                    .args(&["-i", "pipe:0", "-f", "wav", "-ac", "2", "-ar", "48000", "pipe:1"])
                    .stdin(stdout)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .expect("Failed to spawn ffmpeg");

                let container = songbird::input::ChildContainer(vec![ytdl, ffmpeg]);
                let input = songbird::input::Input::from(container);
                let track = songbird::tracks::Track::new_with_data(input, std::sync::Arc::new(title.clone()));
                handler.enqueue(track).await;

                let queue_len = handler.queue().len();
                if queue_len > 1 {
                    ctx.say(format!("🎵 Ditambahkan ke antrean (posisi #{}): **{}**", queue_len, title)).await?;
                } else {
                    ctx.say(format!("▶️ Memutar: **{}**", title)).await?;
                }
            }
            Err(e) => {
                println!("[Music] ERROR resolving URL: {:?}", e);
                ctx.say(format!("❌ Gagal memproses URL: ```{}```", e)).await?;
            }
        }
    } else {
        ctx.say("Bot belum bergabung ke voice channel apapun di server ini. Gunakan `/join` terlebih dahulu!").await?;
    }

    Ok(())
}

/// Menampilkan daftar lagu dalam antrean (Show playlist)
#[poise::command(slash_command, prefix_command)]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("Command ini hanya bisa digunakan di dalam server (Guild).").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue().current_queue();
        
        if queue.is_empty() {
            ctx.say("Antrean saat ini kosong.").await?;
            return Ok(());
        }

        let mut response = String::from("🎶 **Antrean Lagu:**\n");
        for (i, track) in queue.iter().enumerate() {
            // Kita extract title dari `user_data`
            // Dalam songbird 0.6.0, `track.data::<String>()` me-return Arc<String>.
            // Jika data-nya tidak pernah di-set dengan String, ini akan panic, jadi lebih aman tidak di-unwrap langsung,
            // tapi sayangnya API `data<T>` di songbird menggunakan `.expect()`.
            // Jika kita selalu menggunakan `Track::new_with_data` dengan Arc<String>, maka aman.
            
            let title = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                track.data::<String>().to_string()
            })).unwrap_or_else(|_| "Unknown Title".to_string());
            
            if i == 0 {
                response.push_str(&format!("**1.** {} (Sedang diputar)\n", title));
            } else {
                response.push_str(&format!("**{}.** {}\n", i + 1, title));
            }
        }
        
        ctx.say(response).await?;
    } else {
        ctx.say("Bot sedang tidak berada di voice channel.").await?;
    }

    Ok(())
}

/// Menghentikan lagu dan membersihkan antrean
#[poise::command(slash_command, prefix_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("Command ini hanya bisa digunakan di dalam server (Guild).").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        handler.queue().stop();
        ctx.say("🛑 Musik dihentikan dan antrean dibersihkan.").await?;
    } else {
        ctx.say("Bot sedang tidak berada di voice channel.").await?;
    }

    Ok(())
}

/// Melewati lagu yang sedang diputar (Next song)
#[poise::command(slash_command, prefix_command)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("Command ini hanya bisa digunakan di dalam server (Guild).").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let _ = handler.queue().skip();
        ctx.say("⏭️ Lagu dilewati.").await?;
    } else {
        ctx.say("Bot sedang tidak berada di voice channel.").await?;
    }

    Ok(())
}

/// Menghapus semua lagu di antrean / playlist (kecuali yang sedang diputar)
#[poise::command(slash_command, prefix_command)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("Command ini hanya bisa digunakan di dalam server (Guild).").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        handler.queue().modify_queue(|q| {
            q.drain(1..);
        });
        ctx.say("🗑️ Playlist/antrean berhasil dihapus.").await?;
    } else {
        ctx.say("Bot sedang tidak berada di voice channel.").await?;
    }

    Ok(())
}

/// Melompat ke lagu tertentu di dalam antrean (nomor antrean)
#[poise::command(slash_command, prefix_command)]
pub async fn jump(
    ctx: Context<'_>,
    #[description = "Nomor lagu di antrean (contoh: 3)"] index: usize,
) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("Command ini hanya bisa digunakan di dalam server (Guild).").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        
        let q_len = queue.len();
        if index <= 1 || index > q_len {
            ctx.say(format!("Nomor lagu tidak valid. Antrean saat ini memiliki {} lagu. (Gunakan angka > 1)", q_len)).await?;
            return Ok(());
        }
        
        queue.modify_queue(|q| {
            // Hapus lagu dari urutan 1 hingga (index - 1)
            q.drain(1..(index - 1));
        });
        let _ = queue.skip();
        ctx.say(format!("⏭️ Melompat ke lagu nomor {}.", index)).await?;
    } else {
        ctx.say("Bot sedang tidak berada di voice channel.").await?;
    }

    Ok(())
}
