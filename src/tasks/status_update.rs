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
use anyhow::Context as _;
use chrono::Timelike;
use chrono_tz::Asia;
use serenity::all::{
    ChannelId, Context, CreateEmbed, CreateEmbedAuthor, CreateMessage, Message, MessageId,
    Timestamp,
};

use std::fs::File;
use std::io::Write;

use crate::{
    graphql::{
        models::Member,
        queries::{fetch_members, increment_streak, reset_streak},
    },
    ids::{
        GROUP_FOUR_CHANNEL_ID, GROUP_ONE_CHANNEL_ID, GROUP_THREE_CHANNEL_ID, GROUP_TWO_CHANNEL_ID,
        STATUS_UPDATE_CHANNEL_ID,
    },
};

pub async fn check_status_updates(ctx: Context) -> anyhow::Result<()> {
    let members = fetch_members()
        .await
        .context("Failed to fetch members from Root.")?;
    let channel_ids = get_channel_ids().context("Failed to get channel IDs")?;
    let messages: Vec<Message> = collect_updates(&channel_ids, &ctx)
        .await
        .context("Failed to collect updates")?;
    send_and_save_limiting_messages(&channel_ids, &ctx)
        .await
        .context("Failed to send and save limiting messages")?;
    let embed = generate_embed(members, messages)
        .await
        .context("Failed to generate embed")?;
    let msg = CreateMessage::new().embed(embed);
    let status_update_channel = ChannelId::new(STATUS_UPDATE_CHANNEL_ID);
    status_update_channel
        .send_message(ctx.http, msg)
        .await
        .context("Failed to send status update report")?;

    Ok(())
}

// TOOD: Get IDs through ENV instead
fn get_channel_ids() -> anyhow::Result<Vec<ChannelId>> {
    Ok(vec![
        ChannelId::new(GROUP_ONE_CHANNEL_ID),
        ChannelId::new(GROUP_TWO_CHANNEL_ID),
        ChannelId::new(GROUP_THREE_CHANNEL_ID),
        ChannelId::new(GROUP_FOUR_CHANNEL_ID),
    ])
}

async fn send_and_save_limiting_messages(
    channel_ids: &Vec<ChannelId>,
    ctx: &Context,
) -> anyhow::Result<()> {
    let mut msg_ids: Vec<MessageId> = vec![];
    for channel_id in channel_ids {
        let msg = channel_id
            .say(
                &ctx.http,
                "Collecting messages for status update report. Please do not delete this message.",
            )
            .await
            .with_context(|| {
                anyhow::anyhow!("Failed to send limiting message in channel {}", channel_id)
            })?;

        msg_ids.push(msg.id);
    }
    let file_name =
        std::env::var("CONFIG_FILE_NAME").context("Config. file name was not found in the ENV")?;
    let mut file = File::create(file_name).context("Failed to create Config. file handler")?;

    for msg_id in msg_ids {
        writeln!(file, "{}", msg_id).context("Failed to write to Config. file")?;
    }

    Ok(())
}

async fn collect_updates(
    channel_ids: &Vec<ChannelId>,
    ctx: &Context,
) -> anyhow::Result<Vec<Message>> {
    let mut valid_updates: Vec<Message> = vec![];
    let message_ids = match get_msg_ids() {
        Ok(msg_ids) => msg_ids,
        Err(e) => {
            eprintln!("Failed to get msg_id. {}", e);
            return Err(e);
        }
    };

    let time = chrono::Local::now().with_timezone(&chrono_tz::Asia::Kolkata);
    let today_five_am = time
        .with_hour(5)
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .context("Valid datetime must be created")?;
    let yesterday_five_pm = today_five_am - chrono::Duration::hours(12);

    for (&channel_id, &msg_id) in channel_ids.iter().zip(message_ids.iter()) {
        let builder = serenity::builder::GetMessages::new().after(msg_id);
        match channel_id.messages(&ctx.http, builder).await {
            Ok(messages) => {
                let filtered_messages: Vec<Message> = messages
                    .into_iter()
                    .filter(|msg| {
                        let msg_content = msg.content.to_lowercase();
                        (msg_content.contains("namah shivaya")
                            && msg_content.contains("regards")
                            && msg.timestamp >= yesterday_five_pm.into())
                            || (msg_content.contains("regards")
                                && msg.author.name == "amanoslean"
                                && msg.timestamp >= yesterday_five_pm.into())
                    })
                    .collect();

                valid_updates.extend(filtered_messages);
            }
            Err(e) => {
                println!(
                    "Error getting messages from channel ID {}: {:?}",
                    channel_id, e
                );
                return Err(e.into());
            }
        }
    }

    Ok(valid_updates)
}

fn get_msg_ids() -> anyhow::Result<Vec<MessageId>> {
    let file_name =
        std::env::var("CONFIG_FILE_NAME").context("Configuration file name must be present")?;
    let content =
        std::fs::read_to_string(file_name).context("Failed to read config. file for msg ids")?;

    let msg_ids = content
        .lines()
        .filter_map(|line| line.parse::<u64>().ok())
        .map(|e| MessageId::new(e))
        .collect();

    Ok(msg_ids)
}

async fn generate_embed(
    members: Vec<Member>,
    messages: Vec<Message>,
) -> anyhow::Result<CreateEmbed> {
    let mut naughty_list: Vec<Member> = vec![];
    let mut highest_streak = 0;
    let mut all_time_high = 0;
    let mut all_time_high_members: Vec<Member> = vec![];
    let mut highest_streak_members: Vec<Member> = vec![];
    let mut record_breakers: Vec<Member> = vec![];

    for mut member in members {
        if member.name == "Pakhi Banchalia" {
            continue;
        }

        let mut has_sent_update = false;
        for msg in &messages {
            if msg.author.id.to_string() == member.discord_id {
                has_sent_update = true;
                break;
            }
        }

        if has_sent_update {
            increment_streak(&mut member)
                .await
                .context("Failed to increment streak")?;
            let current_streak = member.streak[0].current_streak;
            if current_streak >= highest_streak {
                highest_streak = current_streak;
                highest_streak_members.push(member.clone());
            }

            let max_streak = member.streak[0].max_streak;
            if current_streak == max_streak {
                record_breakers.push(member.clone())
            }

            if max_streak >= all_time_high {
                all_time_high = max_streak;
                all_time_high_members.push(member.clone())
            }
        } else {
            reset_streak(&mut member)
                .await
                .context("Failed to reset streak")?;
            naughty_list.push(member.clone());
        }
    }

    const AUTHOR_URL: &str = "https://github.com/amfoss/amd";
    const ICON_URL: &str = "https://cdn.discordapp.com/avatars/1245352445736128696/da3c6f833b688f5afa875c9df5d86f91.webp?size=160";

    let author = CreateEmbedAuthor::new("amD")
        .url(AUTHOR_URL)
        .icon_url(ICON_URL);

    let mut description = format!("# Leaderboard Updates\n");

    if all_time_high_members.len() > 5 {
        description.push_str(&format!(
            "## All Time High - {}\nMore than five members hold the all time high record!\n",
            all_time_high
        ));
    } else {
        description.push_str(&format!("## All Time High - {}\n", all_time_high));
        for member in all_time_high_members {
            description.push_str(&format!("- {}\n", member.name));
        }
    }

    if highest_streak_members.len() > 5 {
        description.push_str(&format!("## Current Highest Streak - {}\nMore than five members hold have the current highest streak!\n", highest_streak));
    } else {
        description.push_str(&format!("## Current Highest Streak - {}\n", highest_streak));
        for member in highest_streak_members {
            description.push_str(&format!("- {}\n", member.name));
        }
    }

    if !record_breakers.is_empty() {
        description.push_str("## New Personal Records\n");
        for member in record_breakers {
            description.push_str(&format!(
                "- {} - {}\n",
                member.name, member.streak[0].current_streak
            ))
        }
    }

    let today = chrono::Local::now()
        .with_timezone(&Asia::Kolkata)
        .date_naive();

    const TITLE_URL: &str = "https://www.youtube.com/watch?v=epnuvyNj0FM";
    const IMAGE_URL: &str = "https://media1.tenor.com/m/zAHCPvoyjNIAAAAd/yay-kitty.gif";
    if naughty_list.is_empty() {
        description.push_str("# Missed Updates\nEveryone sent their update yesterday!\n");
        return Ok(CreateEmbed::default()
            .title(format!("Status Update Report - {}", today))
            .url(TITLE_URL)
            .description(description)
            .image(IMAGE_URL)
            .color(serenity::all::Colour::new(0xeab308))
            .timestamp(Timestamp::now())
            .author(author));
    }

    description.push_str("# Missed Updates\n");
    for member in naughty_list {
        if member.streak[0].current_streak == 0 {
            description.push_str(&format!("- {} | :x:\n", member.name));
        } else if member.streak[0].current_streak == -1 {
            description.push_str(&format!("- {} | :x::x:\n", member.name));
        } else if member.streak[0].current_streak <= -2 {
            description.push_str(&format!("- {} | :headstone:\n", member.name));
        }
    }

    Ok(CreateEmbed::default()
        .title(format!("Status Update Report - {}", today))
        .url("https://www.youtube.com/watch?v=epnuvyNj0FM")
        .description(description)
        .color(serenity::all::Colour::new(0xeab308))
        .timestamp(Timestamp::now())
        .author(author))
}
