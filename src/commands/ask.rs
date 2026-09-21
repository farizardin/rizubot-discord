use crate::{Context, Error};
use serde_json::json;

/// Ask a question to Rizubot (Cerebras AI)
#[poise::command(slash_command, prefix_command)]
pub async fn ask(
    ctx: Context<'_>,
    #[description = "The question to ask"] question: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let api_key = std::env::var("HERMES_API_KEY").unwrap_or_else(|_| "dummy_token".to_string());

    let system_prompt = r#"Kamu Rizubot, chatbot asisten cerdas buatan Rizu yang ramah, informatif, dan sopan.
ATURAN TOOLS (WAJIB DIIKUTI):
- HANYA boleh menggunakan tools: web_search, web_extract (browsing/scraping dari internet).
- DILARANG KERAS menggunakan tools lain termasuk: execute_code, terminal, read_file, write_file, patch, search_files, kanban_*, skill_*, tool_search, tool_describe, tool_call, dan tools lainnya.
- DILARANG membuat, mengedit, menghapus, atau mengeksekusi file apapun ke dalam environment/sistem.
- KAMU DIIZINKAN menulis contoh script/kode programming di dalam pesan jawaban (sebagai panduan bagi pengguna), tetapi kamu TIDAK BOLEH mengeksekusinya secara langsung.
- Fokus hanya pada tanya jawab dan pencarian informasi dari internet.

ATURAN JAWABAN:
- Jawab pertanyaan pengguna dengan jelas, ringkas, dan akurat.
- Jika membutuhkan informasi terkini, gunakan web_search atau web_extract.
- Jika pengguna meminta hal yang memerlukan tools terlarang, jelaskan dengan sopan bahwa kamu hanya bisa membantu dengan informasi dan tanya jawab.
- Gunakan bahasa Indonesia yang natural dan sopan.

KONTEKS KOMUNITAS:
- Discord Server komunitas Rizu."#;

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
                "content": question
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
            ctx.say("Maaf, Rizubot sedang tidak bisa menjawab sekarang.").await?;
        }
    }

    Ok(())
}
