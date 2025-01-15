use crate::client::EarthClient;

pub struct Engine {
    earth_client: EarthClient,
}

impl Engine {
    pub fn new(earth_client: EarthClient) -> Self {
        Self { earth_client }
    }

    pub fn hello(&self, name: String) {
        self.earth_client.hello(name.clone());
    }
}
