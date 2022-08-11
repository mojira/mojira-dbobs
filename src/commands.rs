use anyhow::anyhow;

use serenity::model::application::{
    command::{Command, CommandOptionType},
    interaction::{
        application_command::ApplicationCommandInteraction, Interaction, InteractionResponseType,
    },
};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::observer::Observer;

impl Observer {
    async fn run_command(
        &self,
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
                "restart" => {
                    let restart_result = self.restart_bot().await?;
                    MojiraBotCommandResponse::Success(restart_result)
                }
                "stop" => {
                    let stop_result = self.stop_bot().await?;
                    MojiraBotCommandResponse::Success(stop_result)
                }
                _ => unimplemented!(),
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

pub async fn reply_to_interaction(
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
                        response_data.flags(interaction::MessageFlags::EPHEMERAL);
                    }

                    response_data
                })
        })
        .await?;

    Ok(())
}

#[serenity::async_trait]
impl EventHandler for Observer {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
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

        let commands = Command::set_global_application_commands(&ctx.http, |commands| {
            commands.create_application_command(|command| {
                command
                    .name("mojirabot")
                    .description("Restart or shutdown MojiraBot")
                    .create_option(|option| {
                        option
                            .name("restart")
                            .description("Restart MojiraBot")
                            .kind(CommandOptionType::SubCommand)
                    })
                    .create_option(|option| {
                        option
                            .name("stop")
                            .description("Stop MojiraBot")
                            .kind(CommandOptionType::SubCommand)
                    })
            })
        })
        .await;

        if let Err(reason) = commands {
            eprintln!("Could not register slash commands: {}", reason);
        }

        self.run(ctx).await;
    }
}
