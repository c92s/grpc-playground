use engine::greeter_server::{Greeter, GreeterServer};
use engine::{HelloEngineReply, HelloEngineRequest};
// use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;
use tonic::{transport::Server, Request, Response, Status};
use tracing::trace;

use crate::engine::Engine;

pub mod engine {
    tonic::include_proto!("engine");
}

pub struct EngineServer {
    cancel: CancellationToken,
}

impl EngineServer {
    pub fn new(addr: String, engine: Engine) -> Self {
        let cancel = CancellationToken::new();
        let cancel_inner = cancel.clone();

        trace!("GreeterServer listening on {}", addr);

        let server = Server::builder()
            .add_service(GreeterServer::new(MyGreeter::new(engine)))
            .serve(addr.parse().unwrap());

        tokio::spawn(async move {
            println!("GreeterServer listening on {}", addr);
            tokio::select! {
                _ = server => {},
                _ = cancel_inner.cancelled() => {},
            }
        });

        Self { cancel }
    }

    pub fn shutdown(&mut self) {
        println!("Shutting down EngineServer...");
        self.cancel.cancel();
        println!("EngineServer shutdown complete.");
    }
}

pub struct MyGreeter {
    engine: Engine,
}

impl MyGreeter {
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn hello_engine(
        &self,
        request: Request<HelloEngineRequest>,
    ) -> Result<Response<HelloEngineReply>, Status> {
        print!("Got a request from {:?} ... ", request.remote_addr());

        self.engine.hello("Forwarding 'Hello' Message".to_string());

        let reply = engine::HelloEngineReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        println!("sending reply: {:?}", reply);

        Ok(Response::new(reply))
    }
}
