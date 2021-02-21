use mongodb::{Client, options::ClientOptions, error::Result };
use mongodb::bson::doc;

use crate::models::Message;

pub struct DB {
    pub client: Client
}

const DB_NAME: &str = "discord-analytics";
const MESSAGE_COLLECTION: &str = "messages";
const GUILD_COLLECTION: &str = "guilds";
const CHANNEL_COLLECTION: &str = "channels";
const CONNECTION_STRING: &str = "mongodb://localhost:27017";

impl DB {
    pub async fn init() -> Result<Self> {
        let mut options = ClientOptions::parse(CONNECTION_STRING).await?;
        options.app_name = Some("discord-analytics".to_string());

        Ok(Self {
            client: Client::with_options(options)?
        })
    }

    pub async fn add_message(&self, entry: Message) -> Result<()> {        
        let doc = doc! {
            "author_id": entry.author_id,
            "channel_id": entry.channel_id,
            "guild_id": entry.guild_id,
            "text": entry.text,
            "timestamp": entry.timestamp
        };

        self.client.database(DB_NAME).collection(MESSAGE_COLLECTION)
            .insert_one(doc, None)
            .await?;

        let guild_query = self.client.database(DB_NAME).collection(GUILD_COLLECTION).find_one(doc! { "_id": entry.guild_id }, None).await;
        let guild_query_result;

        match guild_query {
            Ok(g) => guild_query_result = g,
            Err(e) => {
                println!("error querying for guild {}", e);
                return Err(e);
            }
        }

        match guild_query_result {
            Some(_) => {
                self.client.database(DB_NAME).collection(GUILD_COLLECTION)
                .update_one(doc! { "_id": entry.guild_id }, doc! { "$inc": { "message_count": 1 } }, None)
                .await?;
            },
            None => {
                self.client.database(DB_NAME).collection(GUILD_COLLECTION)
                    .insert_one(doc! { "_id": entry.guild_id, "message_count": 1 }, None)
                    .await?;
            }
        }

        let channel_query = self.client.database(DB_NAME).collection(CHANNEL_COLLECTION).find_one(doc! { "_id": entry.channel_id }, None).await;
        let channel_query_result;
        
        match channel_query {
            Ok(g) => channel_query_result = g,
            Err(e) => {
                println!("error querying for channel {}", e);
                return Err(e);
            }
        }

        match channel_query_result {
            Some(_) => {
                self.client.database(DB_NAME).collection(CHANNEL_COLLECTION)
                .update_one(doc! { "_id": entry.channel_id }, doc! { "$inc": { "message_count": 1 } }, None)
                .await?;
            },
            None => {
                self.client.database(DB_NAME).collection(CHANNEL_COLLECTION)
                    .insert_one(doc! { "_id": entry.channel_id, "message_count": 1 }, None)
                    .await?;
            }
        }

        Ok(())
    }
}