mod server;

use crate::server::Server::*;

use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};

use std::collections::HashMap;

use uuid::Uuid;


fn index(req: &Request, res: &mut Response) {
    res.status_code = String::from("200");
    res.data = String::from("{'json': 'much wow'}");
}

fn create_ticket(req: &Request, res: &mut Response) {        
    res.status_code = String::from("200");
    res.data = req.body.clone();
}

//for now json serialization and deserialization will be handled on a per endpoint basis
//until I get tired of typing it out
fn authenticate(req: &Request, res: &mut Response) {
    let repo = UserRepo::new();
    
    let params: Value = serde_json::from_str(&req.body).unwrap(); //deserializing to untyped for prototyping

    let user = repo.find_by_user_name(params["username"].to_string());    

    if user.password == params["password"].as_str().unwrap() {

        //TODO - generate token and save token data to persistent storage


        res.status_code = String::from("200");
        res.data = format!("{{\"auth_token\": \"{}\"}}", uuid::Uuid::new_v4().to_simple());
    } else {
        res.status_code = String::from("401");
        res.data = String::from(r#"{"error": "Failed to validate user"}"#);
    }        
}

#[async_std::main]
async fn main() {
    let mut my_app = App::new();
    
    //add routes
    my_app.add_route("GET", "/", Box::new(index));
    my_app.add_route("POST", "/ticket", Box::new(create_ticket));
    my_app.add_route("POST", "/auth", Box::new(authenticate));

    my_app.start(8080).await;
}

#[derive(Serialize, Deserialize)]
struct User {    
    user_name: String,
    password: String
}

impl User {
    fn default() -> Self {
        Self {            
            user_name: String::from("bbloom"),
            password: String::from("password123")
        }
    }
}

struct UserRepo {
    users: HashMap<u32, User>
}

impl UserRepo {
    //temporary
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(1, User::default());

        Self {
            users: users
        }
    }

    fn find_one(&self, key: u32) -> &User {
        let value = self.users.get(&key).unwrap();

        return value;
    }

    fn find_by_user_name(&self, user_name: String) -> &User {
        
        let value = self.users.get(&1).unwrap();

        return value;

    }
}