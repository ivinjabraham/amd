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
        client::{Context as SerenityContext, FullEvent}, model::{
        channel::ReactionType,
        gateway::GatewayIntents,
        id::{MessageId, RoleId},
    }
};

type Context<'a> = PoiseContext<'a, Data, Error>;
type Error = Box<dyn std::error::Error + Send + Sync>;

struct Data {
    reaction_roles: HashMap<ReactionType, RoleId>,
}

const ARCHIVE_MESSAGE_ID: u64 = 1298636092886749294;
const ARCHIVE_ROLE_ID: u64 = 1208457364274028574;
const MOBILE_ROLE_ID: u64 = 1298553701094395936;
const SYSTEMS_ROLE_ID: u64 = 1298553801191718944;
const AI_ROLE_ID: u64 = 1298553753523453952;
const RESEARCH_ROLE_ID: u64 = 1298553855474270219;
const DEVOPS_ROLE_ID: u64 = 1298553883169132554;
const WEB_ROLE_ID: u64 = 1298553910167994428;

#[tokio::main]
async fn main(
) -> Result<(), Error>{
    dotenv::dotenv().ok();
    let discord_token = std::env::var("DISCORD_TOKEN").context("'DISCORD_TOKEN' was not found")?;

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

    let mut client = serenity::client::ClientBuilder::new(
        discord_token,
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
    .framework(framework)
    .await
    .context("Failed to create the Serenity client")?;

    client.start().await.context("Error running the bot")?;

    Ok(())
}

fn initialize_data() -> Data {
    let mut data = Data {
        reaction_roles: HashMap::new(),
    };

    let archive_role_id = RoleId::new(ARCHIVE_ROLE_ID);
    let mobile_role_id = RoleId::new(MOBILE_ROLE_ID);
    let systems_role_id = RoleId::new(SYSTEMS_ROLE_ID);
    let ai_role_id = RoleId::new(AI_ROLE_ID);
    let research_role_id = RoleId::new(RESEARCH_ROLE_ID);
    let devops_role_id = RoleId::new(DEVOPS_ROLE_ID);
    let web_role_id = RoleId::new(WEB_ROLE_ID);


    let message_roles = [
        (ReactionType::Unicode("📁".to_string()), archive_role_id),
        (ReactionType::Unicode("📱".to_string()), mobile_role_id),
        (ReactionType::Unicode("⚙️".to_string()), systems_role_id),
        (ReactionType::Unicode("🤖".to_string()), ai_role_id),
        (ReactionType::Unicode("📜".to_string()), research_role_id),
        (ReactionType::Unicode("🚀".to_string()), devops_role_id),
        (ReactionType::Unicode("🌐".to_string()), web_role_id),
   ];

    data.reaction_roles.extend::<HashMap<ReactionType, RoleId>>(message_roles.into());

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
            let message_id = MessageId::new(ARCHIVE_MESSAGE_ID);
            if add_reaction.message_id == message_id && data.reaction_roles.contains_key(&add_reaction.emoji) {
                    if let Some(guild_id) = add_reaction.guild_id {
                        // TODO: Use try_join to await concurrently?
                        if let Ok(member) =
                            guild_id.member(ctx, add_reaction.user_id.unwrap()).await
                        {
                            if let Err(e) = member.add_role(&ctx.http, data.reaction_roles.get(&add_reaction.emoji).expect("Hard coded value verified earlier.")).await {
                                eprintln!("Error: {:?}", e);
                            }
                        }
                    }
                }
        }

        FullEvent::ReactionRemove { removed_reaction } => {
                let message_id = MessageId::new(ARCHIVE_MESSAGE_ID);
                if message_id == removed_reaction.message_id && data.reaction_roles.contains_key(&removed_reaction.emoji) {
                    if let Some(guild_id) = removed_reaction.guild_id {
                        if let Ok(member) = guild_id
                            .member(ctx, removed_reaction.user_id.unwrap())
                            .await
                        {
                            if let Err(e) = member.remove_role(&ctx.http, *data.reaction_roles.get(&removed_reaction.emoji).expect("Hard coded value verified earlier")).await {
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
