use std::fs::File;

use serenity::{Client, all::ApplicationId, prelude::GatewayIntents};

use observer::Observer;

mod commands;
mod observer;

#[derive(serde::Deserialize)]
struct Config {
    pub token: String,
    pub app_id: String,
}

impl Config {
    const FILE_PATH: &str = "config.json";

    fn get() -> anyhow::Result<Self> {
        let json = File::open(Self::FILE_PATH)?;
        let config = serde_json::from_reader(json)?;
        Ok(config)
    } 
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::get()?;

    let mut client = Client::builder(config.token, GatewayIntents::non_privileged())
        .event_handler(Observer::new())
        .application_id(ApplicationId::new(config.app_id.parse()?))
        .await
        .expect("Could not create client");

    client.start().await?;
    Ok(())
}
