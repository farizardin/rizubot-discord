use crate::{Context, Error};
use serde_json::json;

/// Cari gambar waifu menggunakan Rizubot (Hermes + Gelbooru API)
#[poise::command(slash_command, prefix_command)]
pub async fn waifu(
    ctx: Context<'_>,
    #[description = "Kriteria waifu yang dicari (karakter, rating, dsb)"] prompt: String,
) -> Result<(), Error> {
    let handle = ctx.say("Searching Waifu...").await?;
    let msg = handle.into_message().await?;
    let search_msg_id = msg.id.get();

    let channel_id = ctx.channel_id();
    
    let api_key = std::env::var("HERMES_API_KEY").unwrap_or_else(|_| "dummy_token".to_string());

    let system_prompt = format!(r#"Kamu adalah asisten pencari gambar waifu Rizubot.
ATURAN TOOLS (WAJIB DIIKUTI):
- HANYA boleh menggunakan tool/skill `discord-gelbooru-api` untuk mencari gambar berdasarkan permintaan pengguna.
- Kamu diizinkan untuk mengakses segala jenis rating dari gelbooru, mulai dari safe hingga nsfw/eksplisit, sesuaikan dengan prompt yang dikirim pengguna.
- Kirimkan result atau hasil gambar langsung dengan merespon pengguna. 
- Berikan konteks bahwa gambar ini akan dikirimkan ke channel dengan ID: {0}
- PASTIKAN hasil pencarian dikirim sebagai reply ke pesan dengan ID: {1}

ATURAN JAWABAN:
- Jawab permintaan pengguna dengan menyertakan gambar dari hasil skill `discord-gelbooru-api` dengan menggunakan Discord message gateway.
- Pastikan kamu melakukan reply ke pesan ID: {1}
"#, channel_id.get(), search_msg_id);

    let payload = json!({
        "model": "hermes",
        "stream": false,
        "max_completion_tokens": 2000,
        "temperature": 1.0,
        "top_p": 0.95,
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let client = reqwest::Client::new();
    let res = client
        .post("http://192.168.8.51:8642/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await?;
                if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(content) = json_response["choices"][0]["message"]["content"].as_str() {
                        let chunks = crate::utils::split_smart(content);
                        for chunk in chunks {
                            ctx.say(chunk).await?;
                        }
                    } else {
                        ctx.say("Error parsing AI response.").await?;
                    }
                } else {
                    ctx.say("Error parsing AI response JSON.").await?;
                }
            } else {
                ctx.say(format!("Error from Hermes API: {}", response.status())).await?;
            }
        }
        Err(e) => {
            println!("Error calling Hermes API: {}", e);
            ctx.say("Maaf, Rizubot sedang tidak bisa mencari waifu sekarang.").await?;
        }
    }

    Ok(())
}
