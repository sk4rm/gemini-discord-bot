use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use rig::completion::Prompt;
use rig::prelude::*;
use rig::providers::gemini;
use rig::providers::gemini::completion::gemini_api_types::GenerationConfig;
use serde::Serialize;

struct Handler;

#[derive(Serialize)]
struct Params {
    generationConfig: GenerationConfig,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mention = format!("<@{}>", ctx.cache.current_user().id);
        if msg.content.contains(mention.as_str()) {
            let gemini_client = gemini::Client::from_env();
            let agent = gemini_client
                .agent(gemini::completion::GEMINI_2_5_FLASH_PREVIEW_05_20)
                .preamble("Be creative and concise. Answer directly and clearly.")
                .temperature(0.5)
                .additional_params(
                    serde_json::to_value(Params {
                        generationConfig: GenerationConfig {
                            top_k: Some(1),
                            top_p: Some(0.95),
                            candidate_count: Some(1),
                            ..Default::default()
                        },
                    })
                    .unwrap(),
                )
                .build();

            let prompt = msg.content.replace(mention.as_str(), "");
            let response = match agent.prompt(prompt).await {
                Ok(r) => r,
                Err(why) => return println!("Error prompting Gemini: {why:?}."),
            };

            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                println!("Error sending message: {why:?}.");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment.");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client.");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}.");
    }
}
