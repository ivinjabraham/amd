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
mod commands;
mod ids;
mod scheduler;
mod utils;

use crate::ids::{
    AI_ROLE_ID, ARCHIVE_MESSAGE_ID, ARCHIVE_ROLE_ID, DEVOPS_ROLE_ID, MOBILE_ROLE_ID,
    RESEARCH_ROLE_ID, SYSTEMS_ROLE_ID, WEB_ROLE_ID,
};
use anyhow::Context as _;
use poise::{Context as PoiseContext, Framework, FrameworkOptions, PrefixFrameworkOptions};
use serenity::{
    all::{ReactionType, RoleId},
    client::{Context as SerenityContext, FullEvent},
    model::{gateway::GatewayIntents, id::MessageId},
};
use std::collections::HashMap;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = PoiseContext<'a, Data, Error>;

pub struct Data {
    pub reaction_roles: HashMap<ReactionType, RoleId>,
}

pub fn initialize_data() -> Data {
    let mut data = Data {
        reaction_roles: HashMap::new(),
    };

   let roles = [
        (ReactionType::Unicode("📁".to_string()), RoleId::new(ARCHIVE_ROLE_ID)),
        (ReactionType::Unicode("📱".to_string()), RoleId::new(MOBILE_ROLE_ID)),
        (ReactionType::Unicode("⚙️".to_string()), RoleId::new(SYSTEMS_ROLE_ID)),
        (ReactionType::Unicode("🤖".to_string()), RoleId::new(AI_ROLE_ID)),
        (ReactionType::Unicode("📜".to_string()), RoleId::new(RESEARCH_ROLE_ID)),
        (ReactionType::Unicode("🚀".to_string()), RoleId::new(DEVOPS_ROLE_ID)),
        (ReactionType::Unicode("🌐".to_string()), RoleId::new(WEB_ROLE_ID)),
    ];

    data.reaction_roles
        .extend::<HashMap<ReactionType, RoleId>>(roles.into());

    data
}
#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secret_store: shuttle_runtime::SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: commands::get_commands(),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(String::from("$")),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let data = initialize_data();
                scheduler::run_scheduler(ctx.clone()).await;

                Ok(data)
            })
        })
        .build();

    let client = serenity::client::ClientBuilder::new(
        discord_token,
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
    .framework(framework)
    .await
    .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}

async fn event_handler(
    ctx: &SerenityContext,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::ReactionAdd { add_reaction } => {
            let message_id = MessageId::new(ARCHIVE_MESSAGE_ID);
            if add_reaction.message_id == message_id
                && data.reaction_roles.contains_key(&add_reaction.emoji)
            {
                if let Some(guild_id) = add_reaction.guild_id {
                    // TODO: Use try_join to await concurrently?
                    if let Ok(member) = guild_id.member(ctx, add_reaction.user_id.unwrap()).await {
                        if let Err(e) = member
                            .add_role(
                                &ctx.http,
                                data.reaction_roles
                                    .get(&add_reaction.emoji)
                                    .expect("Hard coded value verified earlier."),
                            )
                            .await
                        {
                            eprintln!("Error: {:?}", e);
                        }
                    }
                }
            }
        }

        FullEvent::ReactionRemove { removed_reaction } => {
            let message_id = MessageId::new(ARCHIVE_MESSAGE_ID);
            if message_id == removed_reaction.message_id
                && data.reaction_roles.contains_key(&removed_reaction.emoji)
            {
                if let Some(guild_id) = removed_reaction.guild_id {
                    if let Ok(member) = guild_id
                        .member(ctx, removed_reaction.user_id.unwrap())
                        .await
                    {
                        if let Err(e) = member
                            .remove_role(
                                &ctx.http,
                                *data
                                    .reaction_roles
                                    .get(&removed_reaction.emoji)
                                    .expect("Hard coded value verified earlier"),
                            )
                            .await
                        {
                            eprintln!("Error: {:?}", e);
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}
