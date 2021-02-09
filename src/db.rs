use mongodb::{Client, options::ClientOptions, error::Result};
use mongodb::bson::doc;

use crate::models::Message;

// pub struct DBMessage {
//     pub id: String,
//     pub author_id: String,
//     pub channel_id: String,
//     pub text: String
// }

pub struct DB {
    pub client: Client
}

const DB_NAME: &str = "discord-analytics";
const MESSAGE_COLLECTION: &str = "messages";
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
            "text": entry.text,
            "timestamp": entry.timestamp
        };

        self.client.database(DB_NAME).collection(MESSAGE_COLLECTION)
            .insert_one(doc, None)
            .await?;
        
        Ok(())
    }
}