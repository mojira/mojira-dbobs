use std::env;

use serenity::model::interactions::application_command::ApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::Client;

struct CommandHandler;

async fn verify_user(
    ctx: &Context,
    guild: Option<GuildId>,
    user: &User,
) -> Result<bool, serenity::Error> {
    // (guild, role)
    const ALLOWED_ROLES: &[(u64, u64)] = &[
        // (test server, admin)
        (646317854584471553, 891087368801632286),
    ];

    if let Some(guild) = guild {
        let guild_id = guild.0;

        for (guild, role) in ALLOWED_ROLES.iter() {
            if *guild == guild_id && user.has_role(&ctx.http, *guild, *role).await? {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

async fn run_command(
    ctx: Context,
    command: ApplicationCommandInteraction,
) -> Result<(), serenity::Error> {
    let user_verified = verify_user(&ctx, command.guild_id, &command.user).await?;

    if command.data.name.as_str() != "mojirabot" {
        unimplemented!();
    }

    if !user_verified {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .content("You don't have permission to execute this command.")
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
            })
            .await?;
    } else {
        let content = match command
            .data
            .options
            .get(0)
            .unwrap()
            .value
            .as_ref()
            .unwrap()
            .as_str()
            .unwrap()
        {
            "restart" => "A restart command has been issued to MojiraBot.",
            "stop" => "A stop command has been issued to MojiraBot.",
            _ => unimplemented!(),
        };

        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(content))
            })
            .await?;
    }

    Ok(())
}

#[serenity::async_trait]
impl EventHandler for CommandHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            if let Err(reason) = run_command(ctx, command).await {
                eprintln!("Error while executing command: {:?}", reason);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        eprintln!("{} is connected!", ready.user.name);

        let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands.create_application_command(|command| {
                command
                    .name("mojirabot")
                    .description("Restart or shutdown MojiraBot")
                    .create_option(|option| {
                        option
                            .name("action")
                            .description("The action to take")
                            .kind(application_command::ApplicationCommandOptionType::String)
                            .required(true)
                            .add_string_choice("restart", "restart")
                            .add_string_choice("stop", "stop")
                    })
            })
        })
        .await;

        match commands {
            Ok(commands) => eprintln!(
                "I now have the following global slash commands: {:#?}",
                commands
            ),
            Err(reason) => eprintln!("Could not register slash commands: {}", reason),
        }
    }
}

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
