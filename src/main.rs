mod server;

use crate::server::Server::*;

use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};

use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;

use uuid::Uuid;

use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};

use bcrypt::{DEFAULT_COST, hash, verify};

use futures::Future;

use async_std::task::sleep;

fn index(req: &Request) -> Pin<Box<dyn Future<Output=Response>>>{
    Box::pin(async move {
        let mut res = Response::default();
        res.data = String::from(r#"{"json": "much wow"}"#);

        sleep(Duration::from_secs(5)).await;

        return res;
    })
}

fn create_ticket(req: &Request, res: &mut Response) {        
    res.status_code = String::from("200");
    res.data = req.body.clone();
}

//for now json serialization and deserialization will be handled on a per endpoint basis
//until I get tired of typing it out
fn authenticate(req: &Request, res: &mut Response) {
    // let repo = UserRepo::new();
    
    // let params: Value = serde_json::from_str(&req.body).unwrap(); //deserializing to untyped for prototyping

    // let user = repo.find_by_user_name(params["username"].to_string());    

    // if user.password == params["password"].as_str().unwrap() {

    //     //TODO - generate token and save token data to persistent storage


    //     res.status_code = String::from("200");
    //     res.data = format!("{{\"auth_token\": \"{}\"}}", uuid::Uuid::new_v4().to_simple());
    // } else {
    //     res.status_code = String::from("401");
    //     res.data = String::from(r#"{"error": "Failed to validate user"}"#);
    // }        
}

//Post route for priming mongodb for first time use
//currently only used to setup an admin account
fn install(req: &Request) -> Pin<Box<dyn Future<Output=Response>>>{
    let params: Value = serde_json::from_str(&req.body).unwrap();

    let h_pass = hash(params["password"].as_str().unwrap(), DEFAULT_COST).unwrap();

    let user = User {
        user_name: params["user_name"].as_str().unwrap().to_string(),
        hashed_password: h_pass
    };

    let users = UserRepo::new();
    
    Box::pin(async move {

        let mut res = Response::default();

        users.insert_one(user).await;

        res.status_code = String::from("200");
        res.data = String::from(r#"{"message": "user successfully created"}"#);

        return res;
    })
}

fn async_test(req: &Request) -> Pin<Box<dyn Future<Output=Response>>> {
    Box::pin(async {
        sleep(Duration::from_secs(5)).await;

        return Response::default();
    })
}

#[async_std::main]
async fn main() {
    let mut my_app = App::new();
    
    //add routes
    my_app.add_route("GET", "/", Box::new(index));
    // my_app.add_route("POST", "/ticket", Box::new(create_ticket));
    // my_app.add_route("POST", "/auth", Box::new(authenticate));

    //install routes
    my_app.add_route("POST", "/install", Box::new(install));

    //test routes
    my_app.add_route("GET", "/test", Box::new(async_test));

    my_app.start(8080).await;
}

#[derive(Serialize, Deserialize)]
struct User {    
    user_name: String,
    hashed_password: String
}

impl User {
    fn default() -> Self {
        Self {            
            user_name: String::from("bbloom"),
            hashed_password: String::from("password123")
        }
    }
}

struct MongoConfig {
    connection: String,
    database: String
}

impl MongoConfig {
    fn default() -> Self {
        Self {
            connection: String::from("mongodb://localhost:27017"),
            database: String::from("lab_support_data")
        }
    }
}

struct UserRepo {
    config: MongoConfig
}

impl UserRepo {    
    fn new() -> Self {        
        Self {
            config: MongoConfig::default()
        }
    }

    // fn find_one(&self, key: u32) -> &User {
    //     //TODO
    // }

    // fn find_by_user_name(&self, user_name: String) -> &User {
    //     //TODO
    // }

    async fn insert_one(&self, user: User) {
        let mut client_options = ClientOptions::parse(&self.config.connection).await.unwrap();
        let client = Client::with_options(client_options).unwrap();

        let db = client.database(&self.config.database);

        let collection = db.collection::<User>("Users");        

        collection.insert_one(user, None).await.unwrap();
    }
}