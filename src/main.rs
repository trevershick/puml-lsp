use log::*;
use puml::server;

#[tokio::main]
async fn main() -> puml::Result<()> {
    env_logger::init();
    info!("Creating Server");

    let mut server = server::PlantUmlLanguageServer::new("0.0.0.0:3030");
    info!("Starting Server");
    server.start().await?;
    info!("Stopped.");
    Ok(())
}
