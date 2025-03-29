use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender};

pub mod point {
    tonic::include_proto!("point");
}

use point::point_storage_client::PointStorageClient;
use point::{CreateResponse, Point, ReadResponse};
use tokio_util::sync::CancellationToken;

#[derive(Error, Debug)]
enum PointError {
    #[error("Internal error")]
    Internal,
}

enum PointRequest {
    Create(Point),
    Read(u64),
    Update(u64, Point),
    Delete(u64),
}

enum PointResponse {
    Create(u64),
    Read(Point),
    Update(()),
    Delete(()),
}

trait PointLayer {
    fn create(&mut self, point: Point) -> Result<u64, PointError>;
    fn read(&mut self, id: u64) -> Result<Point, PointError>;
    fn update(&mut self, id: u64, point: Point) -> Result<(), PointError>;
    fn delete(&mut self, id: u64) -> Result<(), PointError>;
}

pub struct PointClient {
    req: Sender<PointRequest>,
    res: Receiver<PointResponse>,
}

impl PointClient {
    pub fn connect(addr: String, cancel: CancellationToken) -> Self {
        // start an os thread for handling all future async client requests
        let (req_tx, mut req_rx) = tokio::sync::mpsc::channel::<PointRequest>(1);
        let (res_tx, res_rx) = tokio::sync::mpsc::channel::<PointResponse>(1);

        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let mut channel = PointStorageClient::connect(addr).await.unwrap();

                // Handle signals in the loop
                loop {
                    tokio::select! {
                        () = cancel.cancelled() => {
                            println!("Cancellation signal received");
                            break;
                        }
                        Some(cmd) = req_rx.recv() => {
                            match cmd {
                                PointRequest::Create(point) => {
                                    let point = Some(point);
                                    let req = point::CreateRequest { point };
                                    let res: CreateResponse = channel.create(req).await.unwrap().into_inner();
                                    res_tx.send(PointResponse::Create(res.id)).await.unwrap();
                                }
                                PointRequest::Read(id) => {
                                    let req = point::ReadRequest { id };
                                    let res: ReadResponse = channel.read(req).await.unwrap().into_inner();
                                    res_tx.send(PointResponse::Read(res.point.unwrap())).await.unwrap();
                                }
                                PointRequest::Update(id, point) => {
                                    let req = point::UpdateRequest { id, point: Some(point) };
                                    let _ = channel.update(req).await.unwrap();
                                    res_tx.send(PointResponse::Update(())).await.unwrap();

                                }
                                PointRequest::Delete(id) => {
                                    let req = point::DeleteRequest { id };
                                    let _ = channel.delete(req).await.unwrap();
                                    res_tx.send(PointResponse::Delete(())).await.unwrap();
                                }
                            }
                        }
                    };
                }
            });
        });
        Self {
            req: req_tx,
            res: res_rx,
        }
    }
}

impl PointLayer for PointClient {
    fn create(&mut self, point: Point) -> Result<u64, PointError> {
        let _ = self.req.blocking_send(PointRequest::Create(point));
        match self.res.blocking_recv() {
            Some(PointResponse::Create(id)) => Ok(id),
            Some(_) => Err(PointError::Internal),
            None => Err(PointError::Internal),
        }
    }

    fn read(&mut self, id: u64) -> Result<Point, PointError> {
        let _ = self.req.blocking_send(PointRequest::Read(id));
        match self.res.blocking_recv() {
            Some(PointResponse::Read(point)) => Ok(point),
            Some(_) => Err(PointError::Internal),
            None => Err(PointError::Internal),
        }
    }

    fn update(&mut self, id: u64, point: Point) -> Result<(), PointError> {
        let _ = self.req.blocking_send(PointRequest::Update(id, point));
        match self.res.blocking_recv() {
            Some(PointResponse::Update(())) => Ok(()),
            Some(_) | None => Err(PointError::Internal),
        }
    }

    fn delete(&mut self, id: u64) -> Result<(), PointError> {
        let _ = self.req.blocking_send(PointRequest::Delete(id));
        match self.res.blocking_recv() {
            Some(PointResponse::Delete(())) => Ok(()),
            Some(_) | None => Err(PointError::Internal),
        }
    }
}

fn main() {
    let cancel = CancellationToken::new();

    let mut client = PointClient::connect("http://[::1]:50051".to_string(), cancel.clone());
    let mut point = Point::default();
    let id: u64 = client.create(point).unwrap();

    for _ in 0..10 {
        point.x += 1.0;
        point.y += 2.0;

        // update the point
        client.update(id, point).unwrap();

        // read the point
        let received_point = client.read(id).unwrap();
        println!("Point: {:?}", received_point);
    }

    // delete the point
    client.delete(id).unwrap();
    println!("Deleted point with id: {}", id);
    cancel.cancel();
}
