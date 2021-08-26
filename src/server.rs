//file for server code
pub mod Server {    

    use async_std::prelude::*;
    use async_std::net::TcpListener;
    use async_std::net::TcpStream;

    use futures::stream::StreamExt;
    use futures::Future;

    use std::collections::HashMap;
    use std::io::Error;
    use std::pin::Pin;

    //todo research why this works and if we are still async
    pub type RequestCallback = dyn Fn(&Connection) -> LabResult;
    pub type MiddlewareCallback = dyn Fn(&Connection) -> LabResult;
    pub type LabResult = Pin<Box<dyn Future<Output=Connection>>>;

    #[derive(Debug, Clone)]
    pub struct Request {
        pub method: String,
        pub path: String,
        pub attributes: HashMap<String, String>,
        pub body: String,
        pub is_authenticated: bool
    }

    impl Request {
        fn default() -> Self {
            Self {
                method: String::from(""),
                path: String::from(""),
                attributes: HashMap::new(),
                body: String::from(""),
                is_authenticated: false
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Connection {
        pub request: Request,
        pub response: Response
    }

    #[derive(Debug, Clone)]
    pub struct Response {
        pub status_code: String,
        pub data: String,
        pub headers: HashMap<String, String>
    }

    impl Response {
        pub fn default() -> Self {
            Self {
                status_code: String::from("200"),
                data: String::from(""),
                headers: HashMap::new()
            }
        }        

        fn build(&self) -> String {
            let mut v: Vec<String> = Vec::new();

            for (key, value) in self.headers.iter() {
                v.push(format!("{}: {}", key, value));
            }

            let mut s = format!("HTTP/1.1 {} OK\r\n", self.status_code);

            for i in v {
                s = format!("{}{}\r\n", s, i);
            }

            s = format!("{}\r\n {}", s, self.data);

            return s;
        }
    }

    pub struct App {
        pub routes: HashMap<String, Box<RequestCallback>>,
        pub middlewares: Vec<Box<MiddlewareCallback>>
    }

    impl App {
        pub fn new() -> Self {
            Self {
                routes: HashMap::new(),
                middlewares: Vec::new()
            }
        }

        pub fn add_route(&mut self, method: &str, path: &str, callback: Box<RequestCallback>) {
            let key = format!("{} {}", method, path);

            self.routes.entry(key).or_insert(callback);
        }

        pub fn add_middleware(&mut self, callback: Box<MiddlewareCallback>) {
            self.middlewares.push(callback);
        }

        async fn handle_connection(&self, mut stream: TcpStream) -> Result<(), Error>{
            let mut buffer = [0; 1024];
        
            stream.read(&mut buffer).await?;    
        
            let req = parse_request(String::from_utf8_lossy(&buffer[..]).to_string());
        
            let response = App::handle_request(&self.routes, &self.middlewares, req);
        
            stream.write_all(response.await.build().as_bytes()).await?;
            stream.flush().await?;
        
            return Ok(());
        }

        //handling request like this is gross. callbacks maybe or [get("/")] [post("/submit")]
        async fn handle_request(routes: &HashMap<String, Box<RequestCallback>>, middlewares: &Vec<Box<MiddlewareCallback>>, request: Request) -> Response {            
            let req = request.clone();
            let res = Response::default();
            
            let mut conn = Connection {
                request: req,
                response: res
            };
            
            //some middleware could probably go here (eg. Auth, Session Management, Logging, etc)
            for middlware in middlewares {
                conn = middlware(&conn).await;
            }
            
            let key = format!("{} {}", conn.request.method, conn.request.path);
            println!("Route Key {}", key);

            let value = routes.get(&key);                        

            if let Some(call_back) = value {
                conn = call_back(&conn).await; //yeah, ASYNC callbacks !!!!!!
            }
            
            return conn.response;
        }

        pub async fn start(&self, port: u32) {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();

            println!("Server started!");                        

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

    fn parse_request(raw_request: String) -> Request {
        println!("{}",raw_request);
        
        let mut request = Request::default();    

        //should split header and body
        let request_vec: Vec<&str> = raw_request.trim_matches(char::from(0)).split("\r\n\r\n").collect();

        //split header
        let header: Vec<&str> = request_vec[0].split("\r\n").collect();
        
        //set the request body
        request.body = request_vec[1].to_string();

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
}