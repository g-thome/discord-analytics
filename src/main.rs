use serenity::prelude::TypeMapKey;
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::{Message as DiscordMessage, MessageType};
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

use dotenv::dotenv;

use db::DB;

use std::env;

use models::Message;

mod db;
mod models;

#[group]
#[commands(ping, stat)]
struct General;

struct Handler;

// struct MongoConnection;

impl TypeMapKey for DB {
    type Value = DB;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: DiscordMessage) {
        if (message.kind != MessageType::Regular) {
            return;
        }
        
        let id = message.id.0;
        let aid = message.author.id.0;
        let cid = message.channel_id.0;
        let txt = message.content.to_string();
        let ts = message.timestamp.to_rfc3339();
        let gid;

        match message.guild_id {
            Some(guild_id) => gid = guild_id.0,
            None => panic!("not found guild id")
        }

        let data = ctx.data.read().await;
        if let Some(db) = data.get::<DB>() {
            let insert = db.add_message(Message {
                _id: id,
                author_id: aid,
                channel_id: cid,
                guild_id: gid,
                text: txt,
                timestamp: ts
            }).await;
    
            match insert {
                Ok(_) => println!("successfully inserted a new document"),
                Err(e) => println!("error on insertion {}", e)
            }
        }        
    }
}

#[command]
async fn ping(ctx: &Context, msg: &DiscordMessage) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn stat(ctx: &Context, msg: &DiscordMessage) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<DB>().unwrap();

    match msg.guild_id {
        Some(guild_id) => {
            let stats = db.guild_stats(guild_id.0).await.expect("something went wrong");
            msg.reply(ctx, format!("this server has {} messages", stats.message_count)).await;
        },
        None => {
            panic!("something went wrong");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    let db = DB::init().await.unwrap();
    
    {
        let mut data = client.data.write().await;
        data.insert::<DB>(db);
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}