use std::collections::HashMap;
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;
pub mod point {
    tonic::include_proto!("point");
}
use crate::point::point_storage_server::{PointStorage, PointStorageServer};
use crate::point::{
    CreateRequest, CreateResponse, DeleteRequest, DeleteResponse, Point, ReadRequest, ReadResponse,
    UpdateRequest, UpdateResponse,
};
use tokio::sync::RwLock;

#[derive(Default)]
pub struct Service {
    storage: RwLock<HashMap<u64, Point>>,
}

#[tonic::async_trait]
impl PointStorage for Service {
    async fn create(
        &self,
        req: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        println!("Received a create request");
        let point = req.into_inner().point.unwrap();
        let id = Uuid::new_v4().as_u64_pair().0;
        self.storage.write().await.insert(id, point);

        println!("Created point with id: {}", id);

        Ok(Response::new(CreateResponse { id }))
    }

    async fn read(&self, req: Request<ReadRequest>) -> Result<Response<ReadResponse>, Status> {
        println!("Received a read request");
        let id: u64 = req.into_inner().id;

        if let Some(point) = self.storage.read().await.get(&id) {
            return Ok(Response::new(ReadResponse {
                point: Some(*point),
            }));
        }
        Err(Status::not_found(format!("Point with id {} not found", id)))
    }

    async fn update(
        &self,
        req: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        println!("Received an update request");
        let req = req.into_inner();
        let id = req.id;
        let point = req.point.unwrap();
        if let Some(existing_point) = self.storage.write().await.get_mut(&id) {
            *existing_point = point;
            return Ok(Response::new(UpdateResponse {}));
        }
        Err(Status::not_found(format!("Point with id {} not found", id)))
    }

    async fn delete(
        &self,
        req: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        println!("Received a delete request");
        self.storage.write().await.remove(&req.into_inner().id);

        Ok(Response::new(DeleteResponse {}))
    }
}

#[tokio::main]
async fn main() {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = Service::default();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(PointStorageServer::new(greeter))
        .serve(addr)
        .await
        .unwrap();
}
