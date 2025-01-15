use tonic::{transport::Server, Request, Response, Status};

use earth::greeter_server::{Greeter, GreeterServer};
use earth::{HelloEarthReply, HelloEarthRequest};

pub mod earth {
    tonic::include_proto!("earth");
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn hello_earth(
        &self,
        request: Request<HelloEarthRequest>,
    ) -> Result<Response<HelloEarthReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = earth::HelloEarthReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
