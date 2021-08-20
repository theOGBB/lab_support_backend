use async_std::prelude::*;
use async_std::net::TcpListener;
use async_std::net::TcpStream;

use futures::stream::StreamExt;

use std::collections::HashMap;
use std::io::Error;

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    attributes: HashMap<String, String>
}

impl Request {
    fn default() -> Self {
        Self {
            method: String::from(""),
            path: String::from(""),
            attributes: HashMap::new()
        }
    }
}

struct Response {
    status_code: String,
    data: String
}

impl Response {
    fn default() -> Self {
        Self {
            status_code: String::from("200"),
            data: String::from("")
        }
    }

    fn build(&self) -> String {
        let s = format!("HTTP/1.1 {} OK\r\n\r\n {}", self.status_code, self.data);

        return s;
    }
}

struct App {
    routes: HashMap<String, Box<dyn Fn(&mut Response)>>
}

impl App {
    fn new() -> Self {
        Self {
            routes: HashMap::new()
        }
    }

    fn add_route(&mut self, method: &str, path: &str, callback: Box<dyn Fn(&mut Response)>) {
        let key = format!("{} {}", method, path);

        self.routes.entry(key).or_insert(callback);
    }

    async fn handle_connection(&self, mut stream: TcpStream) -> Result<(), Error>{
        let mut buffer = [0; 1024];
    
        stream.read(&mut buffer).await?;    
    
        let req = parse_request(String::from_utf8_lossy(&buffer[..]).to_string());
    
        let response = App::handle_request(&self.routes, req);
    
        stream.write_all(response.build().as_bytes()).await?;
        stream.flush().await?;
    
        return Ok(());
    }

    //handling request like this is gross. callbacks maybe or [get("/")] [post("/submit")]
    fn handle_request(routes: &HashMap<String, Box<dyn Fn(&mut Response)>>, req: Request) -> Response {
        let mut res = Response::default();
        res.status_code = String::from("404");

        let key = format!("{} {}", req.method, req.path);
        let value = routes.get(&key);

        if let Some(call_back) = value {
            call_back(&mut res);
        }
        
        return res;
    }

    async fn start(&self, port: u32) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();

        println!("Server started!");

        let mut routes: HashMap<String, Box<dyn Fn(&mut Response)>> = HashMap::new();
        routes.insert(String::from("GET /"), Box::new(index));

        listener
            .incoming()
            .for_each_concurrent(/* limit */ None, |tcpstream| async move {
                let tcpstream = tcpstream.unwrap();
                println!("Someone connected!");
                if let Err(e) = self.handle_connection(tcpstream).await {
                    println!("Error: {}", e);
                }
            })
            .await;
    }
}

#[async_std::main]
async fn main() {
    let mut my_app = App::new();
    
    //add routes
    my_app.add_route("GET", "/", Box::new(index));

    my_app.start(8080).await;
}

fn parse_request(raw_request: String) -> Request {
    let mut request = Request::default();    

    //turn input into array
    let header: Vec<&str> = raw_request.trim_matches(char::from(0)).split("\r\n").collect();

    //get first line
    let first_line: Vec<&str> = header[0].split(" ").collect();        

    if first_line.len() > 1 {
        //set some request values
        request.method = first_line[0].to_string();
        request.path = first_line[1].to_string();

        //parse remaining header info and store in hashmap
        if header.len() > 1 {
            for i in 1..header.len() {
                let parsed_attr: Vec<&str> = header[i].split(":").collect();
        
                if parsed_attr[0].to_string() != "" {
                    request.attributes.insert(parsed_attr[0].to_string(), parsed_attr[1].to_string());
                }
            }
        }    

        println!("{:?}", request);
    }

    return request;
}

fn index(res: &mut Response) {
    res.status_code = String::from("200");
    res.data = String::from("{'json': 'much wow'}");
}