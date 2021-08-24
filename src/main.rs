mod server;
mod repos;

use crate::server::Server::*;
use crate::repos::user_repo::{UserRepo, User};


use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};

use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;

use uuid::Uuid;

use bcrypt::{DEFAULT_COST, hash, verify};

use futures::Future;

use async_std::task::sleep;

fn index(conn: &Connection) -> LabResult {
        
    println!("{}", conn.request.is_authenticated);

    let mut cc = conn.clone();

    Box::pin(async move {        
        cc.response.data = String::from(r#"{"json": "much wow"}"#);

        sleep(Duration::from_secs(5)).await;
        
        return cc;
    })
}

fn create_ticket(conn: &Connection) -> LabResult {

    let mut cc = conn.clone();
    
    Box::pin(async move {
        return cc;
    })
}

//for now json serialization and deserialization will be handled on a per endpoint basis
//until I get tired of typing it out
fn authenticate(conn: &Connection) -> LabResult {
    let mut cc = conn.clone();
    let repo = UserRepo::new();
    
    let params: Value = serde_json::from_str(&conn.request.body).unwrap(); //deserializing to untyped for prototyping    

    Box::pin( async move {
        let user = repo.find_by_user_name(params["user_name"].as_str().unwrap().to_string()).await.unwrap();    
        

        if verify(params["password"].as_str().unwrap(), &user.hashed_password).unwrap() {

            //TODO - generate token and save token data to persistent storage


            cc.response.status_code = String::from("200");
            cc.response.data = format!("{{\"auth_token\": \"{}\"}}", uuid::Uuid::new_v4().to_simple());
        } else {
            cc.response.status_code = String::from("401");
            cc.response.data = String::from(r#"{"error": "Failed to validate user"}"#);
        }
        
        return cc;
    })    
}

//Post route for priming mongodb for first time use
//currently only used to setup an admin account
fn install(conn: &Connection) -> LabResult {
    let mut cc = conn.clone();

    let params: Value = serde_json::from_str(&cc.request.body).unwrap();

    let h_pass = hash(params["password"].as_str().unwrap(), DEFAULT_COST).unwrap();

    let user = User {
        id: Uuid::new_v4().to_simple().to_string(),
        user_name: params["user_name"].as_str().unwrap().to_string(),
        hashed_password: h_pass
    };

    let users = UserRepo::new();
    
    Box::pin(async move {        

        users.insert_one(user).await;

        cc.response.status_code = String::from("200");
        cc.response.data = String::from(r#"{"message": "user successfully created"}"#);

        return cc;
    })
}

fn protect(conn: &Connection) -> LabResult {    
    let mut cc = conn.clone();

    Box::pin(async move {
        cc.request.is_authenticated = true;

        return cc;
    })
}

#[async_std::main]
async fn main() {
    let mut my_app = App::new();
    
    //add routes
    my_app.add_route("GET", "/", Box::new(index));
    my_app.add_route("POST", "/ticket", Box::new(create_ticket));
    my_app.add_route("POST", "/auth", Box::new(authenticate));

    //install routes
    my_app.add_route("POST", "/install", Box::new(install));

    //middlewares
    my_app.add_middleware(Box::new(protect));

    my_app.start(8080).await;
}