mod server;

use crate::server::Server::*;


fn index(req: &Request, res: &mut Response) {
    res.status_code = String::from("200");
    res.data = String::from("{'json': 'much wow'}");
}

fn create_ticket(req: &Request, res: &mut Response) {        
    res.status_code = String::from("200");
    res.data = req.body.clone();
}

#[async_std::main]
async fn main() {
    let mut my_app = App::new();
    
    //add routes
    my_app.add_route("GET", "/", Box::new(index));
    my_app.add_route("POST", "/ticket", Box::new(create_ticket));

    my_app.start(8080).await;
}