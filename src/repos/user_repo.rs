//File contains code needed to connect to mongodb and insert users into
//the collection. Also contains the struct that mimics the user collection for
//strongly typed interface to work with

use super::mongo_config::*;

use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};

use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};


pub struct UserRepo {
    config: MongoConfig
}

impl UserRepo {    
    pub fn new() -> Self {        
        Self {
            config: MongoConfig::default()
        }
    }

    // fn find_one(&self, key: u32) -> &User {
    //     //TODO
    // }

    pub async fn find_by_user_name(&self, user_name: String) -> Option<User> {
        let mut client_options = ClientOptions::parse(&self.config.connection).await.unwrap();
        let client = Client::with_options(client_options).unwrap();

        let db = client.database(&self.config.database);

        let collection = db.collection::<User>("Users"); 

        let filter = doc! {"user_name": user_name};
        println!("{}", filter);

        let value = collection.find_one(filter, None).await.unwrap();

        return value;
    }

    pub async fn insert_one(&self, user: User) {
        let mut client_options = ClientOptions::parse(&self.config.connection).await.unwrap();
        let client = Client::with_options(client_options).unwrap();

        let db = client.database(&self.config.database);

        let collection = db.collection::<User>("Users");        

        collection.insert_one(user, None).await.unwrap();
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,    
    pub user_name: String,
    pub hashed_password: String
}

//would return a list of roles here too
#[derive(Serialize, Deserialize)]
pub struct UserClient {
    pub id: String,
    pub user_name: String,
    pub active_token: String
}

impl User {
    pub fn default() -> Self {
        Self {  
            id: String::from(""),          
            user_name: String::from("bbloom"),
            hashed_password: String::from("password123")
        }
    }
}