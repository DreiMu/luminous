use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use dotenv::dotenv;

use serenity::async_trait;
use serenity::http::CacheHttp;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};
use serenity::utils::MessageBuilder;

mod api;

struct Handler;

struct Unterhaltung;

impl TypeMapKey for Unterhaltung {
    type Value = Arc<RwLock<Vec<(u64, String)>>>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let unterhaltung_lock = {
            let unterhaltung_read = ctx.data.read().await;

            unterhaltung_read.get::<Unterhaltung>().expect("Expected unterhaltung in TypeMap.").clone()
        };
        if msg.channel_id != env::var("CHANNEL_ID").expect("Channel ID").parse::<u64>().expect("Channel ID Not a number") {
            return;
        }
        if msg.author.bot {
            return;
        }
        if msg.content == "!clear" {
            {
                let mut unterhaltung = unterhaltung_lock.write().await;
                unterhaltung.clear();
            }
            
            return;
        }
        let response = {
            let mut unterhaltung = unterhaltung_lock.write().await;
            unterhaltung.push((msg.author.id.0, msg.content.clone()));

            msg.channel_id.broadcast_typing(&ctx.http).await.expect("Error in Typing");

            let response = api::request(unterhaltung.clone(), ctx.clone()).await;

            unterhaltung.push((ctx.cache.current_user().id.0, response.clone()));

            println!("{unterhaltung:?}");
            println!("DATA");
    
            println!("Message: {:?}", msg.content);
            response
        };

        let response = MessageBuilder::new()
            .push(response)
            .build();

        if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
            println!("Error sending message: {why:?}");
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(StandardFramework::new())
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Unterhaltung>(Arc::new(RwLock::new(Vec::new())));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}