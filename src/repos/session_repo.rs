use super::mongo_config::*;

use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};

use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub expiration: String
}

pub struct SessionRepo {
    config: MongoConfig
}

impl SessionRepo {    
    pub fn new() -> Self {        
        Self {
            config: MongoConfig::default()
        }
    }

    // fn find_one(&self, key: u32) -> &User {
    //     //TODO
    // }

    pub async fn find_by_token(&self, token: String) -> Option<Session> {
        let mut client_options = ClientOptions::parse(&self.config.connection).await.unwrap();
        let client = Client::with_options(client_options).unwrap();

        let db = client.database(&self.config.database);

        let collection = db.collection::<Session>("Sessions"); 

        let filter = doc! {"token": token};        

        let value = collection.find_one(filter, None).await.unwrap();

        return value;
    }

    pub async fn insert_one(&self, session: Session) {
        let mut client_options = ClientOptions::parse(&self.config.connection).await.unwrap();
        let client = Client::with_options(client_options).unwrap();

        let db = client.database(&self.config.database);

        let collection = db.collection::<Session>("Sessions");        

        collection.insert_one(session, None).await.unwrap();
    }
}