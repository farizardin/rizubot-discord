use crate::{Context, Error};
use serde_json::json;



/// Dev mode for N8N AI Agent Rizubot
#[poise::command(slash_command, prefix_command)]
pub async fn dev(
    ctx: Context<'_>,
    #[description = "The question to ask N8N"] question: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let n8n_webhook_url = std::env::var("N8N_WEBHOOK_URL_PROD").unwrap_or_default();
    if n8n_webhook_url.is_empty() {
        ctx.say("Error: N8N_WEBHOOK_URL_PROD is not set.").await?;
        return Ok(());
    }

    let payload = json!({
        "message": question
    });

    let client = reqwest::Client::new();
    let res = client
        .post(&n8n_webhook_url)
        .json(&payload)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await?;
                println!("N8N Raw Response: {}", text); // For debugging
                
                if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(&text) {
                    // Try different fields N8N might return ("output" or "text")
                    let output_text = json_response["output"].as_str()
                        .or_else(|| json_response["text"].as_str())
                        .or_else(|| {
                            json_response.as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|first| first["output"].as_str().or(first["text"].as_str()))
                        });

                    if let Some(output) = output_text {
                        let chunks = crate::utils::split_smart(output);
                        for chunk in chunks {
                            ctx.say(chunk).await?;
                        }
                    } else {
                        // Fallback to sending the raw JSON if it doesn't match the expected schema
                        let raw = serde_json::to_string_pretty(&json_response).unwrap_or(text);
                        let msg = format!("**N8N Response (No 'output' or 'text' string field):**\n```json\n{}\n```", raw);
                        let chunks = crate::utils::split_smart(&msg);
                        for chunk in chunks {
                            ctx.say(chunk).await?;
                        }
                    }
                } else {
                    ctx.say("Error parsing N8N response JSON.").await?;
                }
            } else {
                ctx.say(format!("Error from N8N: {}", response.status())).await?;
            }
        }
        Err(e) => {
            println!("Error calling N8N: {}", e);
            ctx.say("Maaf, Rizubot sedang tidak bisa menjawab sekarang.").await?;
        }
    }

    Ok(())
}
