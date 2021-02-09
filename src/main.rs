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

        let user_id = message.author.id;
        let channel_id = message.channel_id;
        let txt = message.content;

        let insert = self.db.add_message(Message {
            author_id: user_id.to_string(),
            channel_id: channel_id.to_string(),
            text: txt.to_string(),
            timestamp: message.timestamp.to_rfc3339()
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