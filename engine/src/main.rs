use crossbeam_channel::{bounded, select, tick, Receiver};
use server::EngineServer;
use std::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt::Layer, EnvFilter};

mod client;
mod engine;
mod server;

use client::EarthClient;
use engine::Engine;

fn ctrl_channel() -> Receiver<()> {
    let (tx, rx) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .unwrap();

    rx
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .from_env_lossy();
    // .add_directive("tokio::task=trace".parse().unwrap())
    // .add_directive("tokio::runtime=trace".parse().unwrap());

    tracing_subscriber::registry()
        .with(console_subscriber::spawn())
        .with(Layer::new().with_writer(std::io::stderr))
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    // start local clients
    let earth_client = EarthClient::connect("http://[::1]:50051".into()).await;

    // setup engine
    let engine = Engine::new(earth_client);

    // start local server
    let mut engine_server = EngineServer::new("[::1]:50053".into(), engine);

    // setup ctrl-c handler
    let ctrl_c_events = ctrl_channel();

    loop {
        select! {
            recv(ctrl_c_events) -> _ => break,
            recv(tick(Duration::from_secs(1))) -> _ => {}
        }
    }

    println!("Ctrl-C received, shutting down...");

    // shutdown gracefully
    engine_server.shutdown();

    Ok(())
}
