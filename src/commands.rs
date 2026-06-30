use anyhow::anyhow;

use serenity::all::{CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::observer::Observer;

impl Observer {
    async fn run_command(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> Result<MojiraBotCommandResponse, anyhow::Error> {
        eprintln!("replying to command from user {}", command.user.tag());

        let user_verified = verify_user(ctx, command.guild_id, &command.user).await?;

        let response = if user_verified {
            match command.data.name.as_str() {
                "mojirabot" => {
                    let subcommand = command
                        .data
                        .options
                        .get(0)
                        .ok_or_else(|| anyhow!("Missing subcommand"))?;

                    match subcommand.name.as_str() {
                        "restart" => {
                            let restart_result = self.restart_bot().await?;
                            MojiraBotCommandResponse::Success(restart_result)
                        }
                        "stop" => {
                            let stop_result = self.stop_bot().await?;
                            MojiraBotCommandResponse::Success(stop_result)
                        }
                        _ => return Err(anyhow!("Unknown subcommand")),
                    }
                }
                "autorestart" => {
                    let subcommand = command
                        .data
                        .options
                        .get(0)
                        .ok_or_else(|| anyhow!("Missing subcommand"))?;

                    match subcommand.name.as_str() {
                        "on" => {
                            self.set_enabled(true).await;
                            MojiraBotCommandResponse::Success("I will now automatically restart MojiraBot when I see that it is offline.")
                        }
                        "off" => {
                            self.set_enabled(false).await;
                            MojiraBotCommandResponse::Success("I will not restart MojiraBot automatically. Please use the `/mojirabot` commands to restart it manually if necessary.")
                        }
                        _ => return Err(anyhow!("Unknown subcommand")),
                    }
                }
                _ => MojiraBotCommandResponse::Error("I don't know this command yet, sorry!"),
            }
        } else {
            MojiraBotCommandResponse::Error("You don't have permission to execute this command.")
        };

        eprintln!("command response: {:?}", response);

        Ok(response)
    }
}

// (guild, role)
const ALLOWED_ROLES: &[(u64, u64)] = &[
    // (test server, admin)
    (646317854584471553, 891087368801632286),
    // (mojira, moderator)
    (647810384031645728, 647812320629882910),
    // (mojira, helper)
    (647810384031645728, 647812604949037056),
];

async fn verify_user(
    ctx: &Context,
    guild: Option<GuildId>,
    user: &User,
) -> Result<bool, anyhow::Error> {
    if let Some(guild_id) = guild {
        for (guild, role) in ALLOWED_ROLES.iter() {
            if *guild == guild_id.get() && user.has_role(&ctx.http, *guild, *role).await? {
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
    async fn send(self, ctx: &Context, command: &CommandInteraction) -> Result<(), anyhow::Error> {
        let (msg, ephemeral) = match self {
            MojiraBotCommandResponse::Success(msg) => (msg, false),
            MojiraBotCommandResponse::Error(msg) => (msg, true),
        };

        reply_to_interaction(ctx, command, msg, ephemeral).await
    }
}

pub async fn reply_to_interaction(
    ctx: &Context,
    command: &CommandInteraction,
    message: &str,
    ephemeral: bool,
) -> Result<(), anyhow::Error> {
    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(message)
                    .ephemeral(ephemeral),
            ),
        )
        .await?;

    Ok(())
}

#[serenity::async_trait]
impl EventHandler for Observer {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match self.run_command(&ctx, &command).await {
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

        let commands = Command::set_global_commands(
            &ctx.http,
            vec![
                CreateCommand::new("mojirabot")
                    .description("Restart or shutdown MojiraBot")
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "restart",
                        "Restart MojiraBot",
                    ))
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "stop",
                        "Stop MojiraBot",
                    )),
                CreateCommand::new("autorestart")
                    .description("Enable or disable autorestart")
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "on",
                        "Enable autorestart",
                    ))
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "off",
                        "Disable autorestart",
                    )),
            ],
        )
        .await;

        if let Err(reason) = commands {
            eprintln!("Could not register slash commands: {}", reason);
        }

        self.run(ctx).await;
    }
}
