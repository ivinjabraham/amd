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
mod graphql;
mod scheduler;
mod tasks;
mod utils;

use anyhow::Context as _;
use std::collections::HashMap;

use poise::{Context as PoiseContext, Framework, FrameworkOptions, PrefixFrameworkOptions};
use serenity::{
    client::Context as SerenityContext,
    client::FullEvent,
    model::{
        channel::ReactionType,
        gateway::GatewayIntents,
        id::{MessageId, RoleId},
    },
};

type Context<'a> = PoiseContext<'a, Data, Error>;
type Error = Box<dyn std::error::Error + Send + Sync>;

struct Data {
    reaction_roles: HashMap<MessageId, (ReactionType, RoleId)>,
}

const ARCHIVE_MESSAGE_ID: u64 = 1295821555586175083;
const ARCHIVE_ROLE_ID: u64 = 1208457364274028574;

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
                crate::scheduler::run_scheduler(ctx.clone()).await;

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

fn initialize_data() -> Data {
    let mut data = Data {
        reaction_roles: HashMap::new(),
    };

    let message_id = MessageId::new(ARCHIVE_MESSAGE_ID);
    let role_id = RoleId::new(ARCHIVE_ROLE_ID);

    data.reaction_roles.insert(
        message_id,
        (ReactionType::Unicode("📁".to_string()), role_id),
    );

    data
}

async fn event_handler(
    ctx: &SerenityContext,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::ReactionAdd { add_reaction } => {
            if let Some((expected_reaction, role_id)) =
                data.reaction_roles.get(&add_reaction.message_id)
            {
                if &add_reaction.emoji == expected_reaction {
                    if let Some(guild_id) = add_reaction.guild_id {
                        // TODO: Use try_join to await concurrently?
                        if let Ok(member) =
                            guild_id.member(ctx, add_reaction.user_id.unwrap()).await
                        {
                            if let Err(e) = member.add_role(&ctx.http, *role_id).await {
                                eprintln!("Error: {:?}", e);
                            }
                        }
                    }
                }
            }
        }

        FullEvent::ReactionRemove { removed_reaction } => {
            if let Some((expected_reaction, role_id)) =
                data.reaction_roles.get(&removed_reaction.message_id)
            {
                if &removed_reaction.emoji == expected_reaction {
                    if let Some(guild_id) = removed_reaction.guild_id {
                        if let Ok(member) = guild_id
                            .member(ctx, removed_reaction.user_id.unwrap())
                            .await
                        {
                            if let Err(e) = member.remove_role(&ctx.http, *role_id).await {
                                eprintln!("Error: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}
