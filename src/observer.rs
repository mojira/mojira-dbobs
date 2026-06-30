use serenity::{all::ActivityData, prelude::Context};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, MutexGuard};

const SCREEN_NAME: &str = ".discordbot-";
const CHECK_SH: &str = "./check.sh";
const RESTART_SH: &str = "./restart.sh";
const STOP_SH: &str = "./stop.sh";

const CHECK_INTERVAL: Duration = Duration::from_secs(30);
const RESTART_INTERVAL: Duration = Duration::from_secs(2 * 60);

pub struct ObserverState {
    last_check: Option<bool>,
    last_restart_time: Option<Instant>,
    enabled: bool,
    ctx: Option<Context>,
}

impl ObserverState {
    pub fn new() -> Self {
        Self {
            last_check: None,
            last_restart_time: None,
            enabled: true,
            ctx: None,
        }
    }
}

pub struct Observer {
    data: Mutex<ObserverState>,
}

impl Observer {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(ObserverState::new()),
        }
    }

    async fn data(&self) -> MutexGuard<'_, ObserverState> {
        self.data.lock().await
    }

    async fn set_ctx(&self, ctx: Context) {
        self.data().await.ctx = Some(ctx);
        self.update_activity().await;
    }

    async fn update_activity(&self) {
        let enabled = self.data().await.enabled;

        if let Some(ctx) = &self.data().await.ctx {
            let activity = if enabled {
                ActivityData::watching("MojiraBot")
            } else {
                ActivityData::listening("commands")
            };
            ctx.set_activity(Some(activity));
        }
    }

    pub async fn set_enabled(&self, enabled: bool) {
        self.data().await.enabled = enabled;
        self.update_activity().await;
    }

    fn run_sh_file(name: &str) -> Result<String, anyhow::Error> {
        let output = std::process::Command::new("sh").arg(name).output()?;
        String::from_utf8(output.stdout).map_err(Into::into)
    }

    /// Checks whether the bot is online.
    async fn is_bot_online(&self) -> Result<bool, anyhow::Error> {
        let output = Self::run_sh_file(CHECK_SH)?;
        Ok(output.contains(SCREEN_NAME))
    }

    pub async fn restart_bot(&self) -> Result<&'static str, anyhow::Error> {
        self.data().await.last_check = Some(true);
        self.data().await.last_restart_time = Some(Instant::now());

        Self::run_sh_file(RESTART_SH)?;
        Ok("A restart command has been issued to MojiraBot.")
    }

    pub async fn stop_bot(&self) -> Result<&'static str, anyhow::Error> {
        self.set_enabled(false).await;

        Self::run_sh_file(STOP_SH)?;
        Ok("A stop command has been issued to MojiraBot.")
    }

    async fn restart_if_necessary(&self) -> Result<bool, anyhow::Error> {
        if self.data().await.enabled {
            let restart_cooldown_over = self
                .data
                .lock()
                .await
                .last_restart_time
                .map_or(true, |last_restart| {
                    last_restart.elapsed() >= RESTART_INTERVAL
                });

            let bot_was_online = self.data().await.last_check.unwrap_or(false);
            let bot_is_online = self.is_bot_online().await?;

            if restart_cooldown_over && !bot_was_online {
                if !bot_is_online {
                    let _ = self.restart_bot().await?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub async fn run(&self, ctx: Context) {
        self.set_ctx(ctx).await;

        let mut interval = tokio::time::interval(CHECK_INTERVAL);
        loop {
            let result = self.restart_if_necessary().await;
            match result {
                Ok(true) => eprintln!("Bot downtime has been detected; restart command has been issued automatically by mojira-dbobs"),
                Ok(false) => {},
                Err(err) => eprintln!("Error while checking MojiraBot: {}", err),
            }
            interval.tick().await;
        }
    }
}
