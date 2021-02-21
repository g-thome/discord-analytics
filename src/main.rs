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
#[commands(ping)]
struct General;

struct Handler {
    db: db::DB
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, message: DiscordMessage) {
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

        let insert = self.db.add_message(Message {
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

#[command]
async fn ping(ctx: &Context, msg: &DiscordMessage) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn stat(ctx: &Context, msg: &DiscordMessage) -> CommandResult {
    let msg_count = 10;
    msg.reply(ctx, format!("this guild has {} messages ", msg_count.to_string()));

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

    let event_handler = Handler {
        db: DB::init().await.expect("could not connect to database")
    };

    let mut client = Client::builder(token)
        .event_handler(event_handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}