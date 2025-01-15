use engine::greeter_client::GreeterClient as EngineGreeterClient;
use tokio::runtime::Runtime;

pub mod engine {
    tonic::include_proto!("engine");
}

pub struct EngineClient {
    client: EngineGreeterClient<tonic::transport::Channel>,
    rt: Runtime,
}

impl EngineClient {
    pub fn connect(addr: String) -> Result<Self, Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let client = rt.block_on(EngineGreeterClient::connect(addr))?;
        Ok(Self { client, rt })
    }

    pub fn hello(&mut self, name: String) -> String {
        let req = engine::HelloEngineRequest { name };
        let res = self.rt.block_on(self.client.hello_engine(req)).unwrap();
        res.into_inner().message
    }
}

fn main() {
    loop {
        let mut client = match EngineClient::connect("http://[::1]:50053".into()) {
            Ok(client) => client,
            Err(e) => {
                println!("Failed to connect to Engine: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
        };

        loop {
            println!("Engine says: {}", client.hello("Engine".into()));
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
