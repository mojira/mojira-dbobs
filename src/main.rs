use std::env;

use serenity::Client;

mod commands;
use commands::CommandHandler;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a discord token in environment. Start this binary with `DISCORD_TOKEN=<token>` in order to use mojira-dbobs.");

    let client_id: u64 = env::var("DISCORD_CLIENT_ID")
        .expect("Expected a discord user id in environment. Start this binary with `DISCORD_USER_ID=<id>` in order to use mojira-dbobs.")
        .parse()
        .expect("Discord user id is not a valid id");

    let mut client = Client::builder(token)
        .event_handler(CommandHandler)
        .application_id(client_id)
        .await
        .expect("Could not create client");

    if let Err(reason) = client.start().await {
        eprintln!("Client error: {}", reason);
    }
}
