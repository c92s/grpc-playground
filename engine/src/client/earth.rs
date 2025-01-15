use earth::greeter_client::GreeterClient as EarthGreeterClient;
use tracing::trace;

macro_rules! call_sync {
    ($code:expr) => {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on($code))
    };
}

pub mod earth {
    tonic::include_proto!("earth");
}

pub struct EarthClient {
    client: EarthGreeterClient<tonic::transport::Channel>,
}

impl EarthClient {
    pub async fn connect(addr: String) -> Self {
        let client = EarthGreeterClient::connect(addr).await.unwrap();

        Self { client }
    }

    pub fn hello(&self, name: String) -> String {
        trace!("Sending HelloEarthRequest ...");
        let req = earth::HelloEarthRequest { name };
        let res = call_sync!(self.client.clone().hello_earth(req)).unwrap();

        trace!("Received HelloEarthReply ...");
        res.into_inner().message
    }
}
