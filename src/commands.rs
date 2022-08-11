use anyhow::anyhow;

use serenity::model::interactions::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub struct CommandHandler;

// (guild, role)
const ALLOWED_ROLES: &[(u64, u64)] = &[
    // (test server, admin)
    (646317854584471553, 891087368801632286),
    // (mojira, moderator)
    (647810384031645728, 647812320629882910),
    // (mojira, helper)
    (647810384031645728, 647812604949037056),
];

const RESTART_SH: &str = "./restart.sh";
const STOP_SH: &str = "./stop.sh";

async fn verify_user(
    ctx: &Context,
    guild: Option<GuildId>,
    user: &User,
) -> Result<bool, anyhow::Error> {
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

#[derive(Debug)]
enum MojiraBotCommandResponse {
    Success(&'static str),
    Error(&'static str),
}

impl MojiraBotCommandResponse {
    async fn send(
        self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), anyhow::Error> {
        let (msg, ephermal) = match self {
            MojiraBotCommandResponse::Success(msg) => (msg, false),
            MojiraBotCommandResponse::Error(msg) => (msg, true),
        };

        reply_to_interaction(ctx, command, msg, ephermal).await
    }
}

fn run_sh_file(name: &str) -> Result<(), anyhow::Error> {
    let output = std::process::Command::new("sh").arg(name).output()?;

    eprintln!("{} output: {:?}", name, output);
    Ok(())
}

async fn restart_command() -> Result<MojiraBotCommandResponse, anyhow::Error> {
    run_sh_file(RESTART_SH)?;

    Ok(MojiraBotCommandResponse::Success(
        "A restart command has been issued to MojiraBot.",
    ))
}

async fn stop_command() -> Result<MojiraBotCommandResponse, anyhow::Error> {
    run_sh_file(STOP_SH)?;

    Ok(MojiraBotCommandResponse::Success(
        "A stop command has been issued to MojiraBot.",
    ))
}

async fn reply_to_interaction(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    message: &str,
    ephermal: bool,
) -> Result<(), anyhow::Error> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    let response_data = data.content(message);

                    if ephermal {
                        response_data
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
                    }

                    response_data
                })
        })
        .await?;

    Ok(())
}

async fn run_command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<MojiraBotCommandResponse, anyhow::Error> {
    eprintln!("replying to command from user {}", command.user);

    let user_verified = verify_user(ctx, command.guild_id, &command.user).await?;

    let response = if command.data.name.as_str() != "mojirabot" {
        MojiraBotCommandResponse::Error("I don't know this command yet, sorry!")
    } else if user_verified {
        let subcommand = command
            .data
            .options
            .get(0)
            .ok_or_else(|| anyhow!("Missing subcommand"))?;

        match subcommand.name.as_str() {
            "restart" => restart_command().await?,
            "stop" => stop_command().await?,
            _ => unimplemented!(),
        }
    } else {
        MojiraBotCommandResponse::Error("You don't have permission to execute this command.")
    };

    eprintln!("command response: {:?}", response);

    Ok(response)
}

#[serenity::async_trait]
impl EventHandler for CommandHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match run_command(&ctx, &command).await {
                Ok(response) => {
                    match response.send(&ctx, &command).await {
                        Ok(_) => {
                            eprintln!("Command successfully replied to.")
                        }
                        Err(reason) => {
                            eprintln!("Error while sending reply: {:?}", reason)
                        }
                    };
                }
                Err(reason) => {
                    eprintln!("Error while executing command: {:?}", reason);
                }
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
                            .name("restart")
                            .description("Restart MojiraBot")
                            .kind(ApplicationCommandOptionType::SubCommand)
                    })
                    .create_option(|option| {
                        option
                            .name("stop")
                            .description("Stop MojiraBot")
                            .kind(ApplicationCommandOptionType::SubCommand)
                    })
            })
        })
        .await;

        if let Err(reason) = commands {
            eprintln!("Could not register slash commands: {}", reason)
        }
    }
}
