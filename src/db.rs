use mongodb::{Client, options::ClientOptions, error::Result, error::Error };
use mongodb::bson::{doc, Bson};
use serenity::http::client::Http;

use crate::models::{ Message, ChannelStats };

use std::env;

pub struct DB {
    pub mongo_client: Client,
    pub http_client: Http
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

        let token = env::var("DISCORD_TOKEN").expect("discord token missing");

        let http_client = Http::new_with_token(&token);

        Ok(Self {
            mongo_client: Client::with_options(options)?,
            http_client: http_client
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

        self.mongo_client.database(DB_NAME).collection(MESSAGE_COLLECTION)
            .insert_one(doc, None)
            .await?;

        let guild_query = self.mongo_client.database(DB_NAME).collection(GUILD_COLLECTION).find_one(doc! { "_id": entry.guild_id }, None).await;
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
                self.mongo_client.database(DB_NAME).collection(GUILD_COLLECTION)
                .update_one(doc! { "_id": entry.guild_id }, doc! { "$inc": { "message_count": 1 } }, None)
                .await?;
            },
            None => {
                self.mongo_client.database(DB_NAME).collection(GUILD_COLLECTION)
                    .insert_one(doc! { "_id": entry.guild_id, "message_count": 1 }, None)
                    .await?;
            }
        }

        let channel_query = self.mongo_client.database(DB_NAME).collection(CHANNEL_COLLECTION).find_one(doc! { "_id": entry.channel_id }, None).await;
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
                self.mongo_client.database(DB_NAME).collection(CHANNEL_COLLECTION)
                .update_one(doc! { "_id": entry.channel_id }, doc! { "$inc": { "message_count": 1 } }, None)
                .await?;
            },
            None => {
                let channel = self.http_client.get_channel(entry.channel_id).await.expect("error in channel API");
                
                match channel.guild() {
                    Some(c) => { 
                        let channel_name = c.name();

                        let channel_document = doc! {
                            "_id": entry.channel_id,
                            "name": channel_name,
                            "message_count": 1
                        };
        
                        self.mongo_client.database(DB_NAME).collection(CHANNEL_COLLECTION)
                            .insert_one(channel_document, None)
                            .await?;
                    },
                    None => println!("can't track private conversations")
                }
                
                
            }
        }

        Ok(())
    }

    pub async fn channel_stats(&self, channel_id: u64) -> Result<ChannelStats> {
        let query = doc! { "_id": channel_id};
        let query_result = self
                        .mongo_client
                        .database(DB_NAME)
                        .collection(CHANNEL_COLLECTION)
                        .find_one(query, None)
                        .await
                        .expect("error trying to query for a channel");

        match query_result {
            Some(channel) => {
                let message_count_as_i64 = channel.get_i64("message_count").unwrap();
                return Ok(ChannelStats {
                    message_count: message_count_as_i64 as u64
                })
            },
            None => println!("channel not found")
        }
    }
}