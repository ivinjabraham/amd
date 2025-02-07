/*
amFOSS Daemon: A discord bot for the amFOSS Discord server.
Copyright (C) 2024 amFOSS

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use tracing::{info, trace};
use tracing_subscriber::EnvFilter;

use crate::{Context, Data, Error};

#[poise::command(prefix_command)]
async fn amdctl(ctx: Context<'_>) -> Result<(), Error> {
    trace!("Running amdctl command");
    ctx.say("amD is up and running.").await?;
    Ok(())
}

#[poise::command(prefix_command, owners_only)]
async fn set_log_level(ctx: Context<'_>, level: String) -> Result<(), Error> {
    trace!("Running set_log_level command");
    let data = ctx.data();
    let reload_handle = data.log_reload_handle.write().await;

    let new_filter = match level.to_lowercase().as_str() {
        "trace" => "trace",
        "debug" => "debug",
        "info" => "info",
        "warn" => "warn",
        "error" => "error",
        _ => {
            ctx.say("Invalid log level! Use: trace, debug, info, warn, error")
                .await?;
            return Ok(());
        }
    };

    if reload_handle.reload(EnvFilter::new(new_filter)).is_ok() {
        ctx.say(format!("Log level changed to **{}**", new_filter))
            .await?;
        info!("Log level changed to {}", new_filter);
    } else {
        ctx.say("Failed to update log level.").await?;
    }

    Ok(())
}

/// Returns a vector containg [Poise Commands][`poise::Command`]
pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
    vec![amdctl(), set_log_level()]
}
