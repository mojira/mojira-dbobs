<!-- shields -->
[![](https://img.shields.io/github/issues/mojira/mojira-dbobs)](https://github.com/mojira/mojira-dbobs/issues)

# mojira-dbobs

<br/>
<p align="center">
  <a href="https://bugs.mojang.com/">
    <img src="mojira-dbobs.png" alt="mojira-dbobs" width="80" height="80">
  </a>

  <h3 align="center">Mojira Discord Bot Observer</h3>

  <p align="center">
    A simple Discord bot for observing and restarting MojiraBot.
  </p>
</p>

## About

This bot has two functions:
* It adds the slash command `/mojirabot` which allows helpers and moderators to stop and restart MojiraBot directly via Discord.
* It checks whether MojiraBot is online in regular intervals, and if it is not, it restarts it automatically.
  This behaviour can be toggled with the `/autorestart` Discord command.

## Usage

This bot is written in Rust using the [Serenity](https://github.com/serenity-rs/serenity/) Discord API library.

For development, use the Rust build tool `cargo`.

* Run for development: `cargo run`
* Build for production: `cargo build --release`

The bot needs the two environment variables `DISCORD_TOKEN` and `DISCORD_USER_ID` for Discord API access.

## Deployment

mojira-dbobs is not deployed automatically. For manual deployment, only the following files need to be transferred to the server:
* All `.sh` files (if they changed)
* The executable file built using `cargo build --release --target x86_64-unknown-linux-gnu` (for 64-bit Linux).
  After running the command, the executable is located in `target/release/mojira-dbobs`.
  (Note that you can also run `strip` to reduce the size of this executable file, but this is optional.)
