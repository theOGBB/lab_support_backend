mod server;
mod repos;

use crate::server::Server::*;
use crate::repos::user_repo::{UserRepo, User};
use crate::repos::session_repo::{Session, SessionRepo};

use chrono::{DateTime, Local, Duration};

use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};

use std::collections::HashMap;
use std::pin::Pin;

use uuid::Uuid;

use bcrypt::{DEFAULT_COST, hash, verify};

use futures::Future;

use async_std::task::sleep;

fn index(conn: &Connection) -> LabResult {            
    let mut cc = conn.clone();    

    Box::pin(async move {
        
        if cc.request.is_authenticated {
            cc.response.data = String::from(r#"{"json": "much wow"}"#);
            cc.response.status_code = String::from("200");
        } else {
            cc.response.status_code = String::from("401");
            cc.response.data = String::from(r#"{"error": "invalid authorization token"}"#);
        }   
        
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

            let token = uuid::Uuid::new_v4().to_simple().to_string();
            let session = Session {
                token: token.clone(),
                user_id: user.id,
                expiration: (Local::now() + Duration::days(1)).to_rfc2822()
            };

            let session_repo = SessionRepo::new();
            session_repo.insert_one(session).await;

            cc.response.status_code = String::from("200");
            cc.response.data = format!("{{\"auth_token\": \"{}\"}}", token);
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

//basic middleware that checks the request for a valid bearer token
//if one is found the request is set as authenticated
fn protect(conn: &Connection) -> LabResult {    
    let mut cc = conn.clone();

    Box::pin(async move {        
        
        if let Some(auth_header) = cc.request.attributes.get("Authorization") {
            let token = auth_header.replace("Bearer", "").trim().to_string();

            println!("Auth token: {}", token);

            let session = SessionRepo::new();            

            if let Some(active_session) = session.find_by_token(token).await {                
                if DateTime::parse_from_rfc2822(&active_session.expiration).unwrap() > Local::now() {
                    //found valid session
                    cc.request.is_authenticated = true;
                }
                else {
                    //found expired session
                    cc.request.is_authenticated = false;
                }
            } else {
                //no session found
                cc.request.is_authenticated = false;
            }            
        } else {
            //no auth token found in request
            cc.request.is_authenticated = false;
        }                
        
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