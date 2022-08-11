use std::{env, sync::Arc};

use serenity::{prelude::GatewayIntents, Client};
use tokio::sync::Mutex;

use commands::CommandHandler;
use observer::{Observer, ObserverService};

mod commands;
mod observer;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a discord token in environment. Start this binary with `DISCORD_TOKEN=<token>` in order to use mojira-dbobs.");

    let client_id: u64 = env::var("DISCORD_CLIENT_ID")
        .expect("Expected a discord user id in environment. Start this binary with `DISCORD_USER_ID=<id>` in order to use mojira-dbobs.")
        .parse()
        .expect("Discord user id is not a valid id");

    let observer = Arc::new(Mutex::new(Observer::new()));

    let mut client = Client::builder(token, GatewayIntents::non_privileged())
        .event_handler(CommandHandler::new(observer.clone()))
        .application_id(client_id)
        .await
        .expect("Could not create client");

    if let Err(reason) = client.start().await {
        eprintln!("Client error: {}", reason);
    }

    let service = ObserverService::new(observer);
    service.run().await;
}
