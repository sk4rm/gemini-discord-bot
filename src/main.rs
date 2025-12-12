use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use rig::completion::Prompt;
use rig::prelude::*;
use rig::providers::gemini;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mention = format!("<@{}>", ctx.cache.current_user().id);
        if msg.content.contains(mention.as_str()) {
            let gemini_client = gemini::Client::from_env();
            let agent = gemini_client
                .agent(gemini::completion::GEMINI_2_5_FLASH)
                .preamble("Answer concisely, directly and clearly.")
                .build();

            let prompt = msg.content.replace(mention.as_str(), "");
            let response = match agent.prompt(prompt).await {
                Ok(response) => response,
                Err(why) => return println!("Error prompting Gemini: {why:?}."),
            };

            let mut chunk = String::new();
            for char in response.chars() {
                chunk.push(char);
                if chunk.len() >= 2000 {
                    if let Err(why) = msg.reply(&ctx.http, chunk.as_str()).await {
                        println!("Error sending message: {why:?}.");
                        break;
                    }
                    chunk.clear();
                }
            }
            if !chunk.is_empty() {
                msg.reply(&ctx.http, chunk.as_str()).await.ok();
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable not set.");
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
