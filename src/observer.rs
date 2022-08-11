use std::{
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

const RESTART_SH: &str = "./restart.sh";
const STOP_SH: &str = "./stop.sh";

const CHECK_INTERVAL: Duration = Duration::from_secs(30);
const RESTART_INTERVAL: Duration = Duration::from_secs(2 * 60);

pub struct Observer {
    last_check: Option<bool>,
    last_restart_time: Option<Instant>,
    enabled: bool,
}

impl Observer {
    pub fn new() -> Self {
        Self {
            last_check: None,
            last_restart_time: None,
            enabled: true,
        }
    }

    fn run_sh_file(name: &str) -> Result<(), anyhow::Error> {
        let _output = std::process::Command::new("sh").arg(name).output()?;
        Ok(())
    }

    pub async fn restart_bot(&mut self) -> Result<&'static str, anyhow::Error> {
        self.last_check = Some(true);
        self.last_restart_time = Some(Instant::now());

        Self::run_sh_file(RESTART_SH)?;
        Ok("A restart command has been issued to MojiraBot.")
    }

    pub async fn stop_bot(&mut self) -> Result<&'static str, anyhow::Error> {
        self.enabled = false;

        Self::run_sh_file(STOP_SH)?;
        Ok("A stop command has been issued to MojiraBot.")
    }

    async fn restart_if_necessary(&mut self) -> Result<bool, anyhow::Error> {
        if self.enabled {
            let restart_cooldown_over = self.last_restart_time.map_or(true, |last_restart| {
                last_restart.elapsed() >= RESTART_INTERVAL
            });

            let bot_was_online = self.last_check.unwrap_or(false);
            let bot_is_online = self.is_bot_online();

            if restart_cooldown_over && !bot_was_online {
                if let Some(false) = bot_is_online {
                    let _ = self.restart_bot().await?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Checks whether the bot is online. If the check fails, returns [None].
    fn is_bot_online(&mut self) -> Option<bool> {
        let mut command = Command::new("screen");
        command.arg("-ls");

        let output = command.output().ok()?;
        let stdout = String::from_utf8(output.stdout).ok()?;

        let result = stdout.contains(".mojiradiscordbot-");
        self.last_check = Some(result);
        Some(result)
    }
}

pub struct ObserverService {
    observer: Arc<Mutex<Observer>>,
    stop: bool,
}

impl ObserverService {
    pub fn new(observer: Arc<Mutex<Observer>>) -> Self {
        Self {
            observer,
            stop: false,
        }
    }

    pub async fn run(&self) {
        let mut interval = tokio::time::interval(CHECK_INTERVAL);
        while !self.stop {
            let result = self.observer.lock().await.restart_if_necessary().await;
            match result {
                Ok(true) => eprintln!("Bot downtime has been detected; restart command has been issued automatically by mojira-dbobs"),
                Ok(false) => {},
                Err(err) => eprintln!("Error while checking MojiraBot: {}", err),
            }
            interval.tick().await;
        }
    }
}
