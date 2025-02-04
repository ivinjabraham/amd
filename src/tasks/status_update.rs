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
use serenity::all::{
    self, ChannelId, Context, CreateEmbed, CreateEmbedAuthor, CreateMessage, Embed,
    Member as DiscordMember, Message, MessageId, Timestamp,
};
use anyhow::Result;

use crate::{
    graphql::{queries::{fetch_members, increment_streak, reset_streak}, models:: Member},
    ids::{
        GROUP_FOUR_CHANNEL_ID, GROUP_ONE_CHANNEL_ID, GROUP_THREE_CHANNEL_ID, GROUP_TWO_CHANNEL_ID,
        STATUS_UPDATE_CHANNEL_ID,
    },
    utils::time::get_five_am_timestamp,
};
use std::fs::File;
use std::{collections::HashMap, io::Write, str::FromStr};

use chrono_tz::Asia;

pub async fn check_status_updates(ctx: Context) -> Result<()> {
    let mut members = match fetch_members().await {
        Ok(members) => members,
        Err(e) => {
            eprintln!("Failed to fetch members from Root. {}", e);
            return Err(e);
        }
    };

    let channel_ids: Vec<ChannelId> = vec![
        ChannelId::new(GROUP_ONE_CHANNEL_ID),
        ChannelId::new(GROUP_TWO_CHANNEL_ID),
        ChannelId::new(GROUP_THREE_CHANNEL_ID),
        ChannelId::new(GROUP_FOUR_CHANNEL_ID),
    ];

    let messages: Vec<Message> = collect_updates(&channel_ids, &ctx).await;
    send_and_save_limiting_messages(&channel_ids, &ctx).await;

    let embed = generate_embed(members, messages, &ctx).await;

    let msg = CreateMessage::new().embed(embed);
    let status_update_channel = ChannelId::new(STATUS_UPDATE_CHANNEL_ID);
    match status_update_channel.send_message(ctx.http, msg).await {
        Err(e) => eprintln!("{}", e),
        _ => (),
    };

    Ok(())
}

async fn send_and_save_limiting_messages(channel_ids: &Vec<ChannelId>, ctx: &Context) {
    let mut msg_ids: Vec<MessageId> = vec![];
    for channel_id in channel_ids {
        let msg = channel_id
            .say(
                &ctx.http,
                "Collecting messages for status update report. Please do not delete this message.",
            )
            .await
            .unwrap();
        msg_ids.push(msg.id);
    }
    let file_name =
        std::env::var("CONFIG_FILE_NAME").expect("Configuration file name must be present");
    let mut file = File::create(file_name).unwrap(); // Create or overwrite the file

    for msg_id in msg_ids {
        writeln!(file, "{}", msg_id).unwrap(); // Write each message ID to a new line
    }
}

async fn collect_updates(channel_ids: &Vec<ChannelId>, ctx: &Context) -> Vec<Message> {
    let mut valid_updates: Vec<Message> = vec![];
    let message_ids = get_msg_ids();

    let time = chrono::Local::now().with_timezone(&chrono_tz::Asia::Kolkata);
    let today_five_am = get_five_am_timestamp(time);
    let yesterday_five_pm = today_five_am - chrono::Duration::hours(12);

    for (&channel_id, &msg_id) in channel_ids.iter().zip(message_ids.iter()) {
        let builder = serenity::builder::GetMessages::new().after(msg_id);
        match channel_id.messages(&ctx.http, builder).await {
            Ok(messages) => {
                for msg in &messages {
                    println!("{:?}", msg);
                }
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
            Err(e) => println!("ERROR: {:?}", e),
        }
    }

    valid_updates
}

fn get_msg_ids() -> Vec<MessageId> {
    let file_name =
        std::env::var("CONFIG_FILE_NAME").expect("Configuration file name must be present");
    let content = std::fs::read_to_string(file_name).unwrap();

    let msg_ids = content
        .lines()
        .filter_map(|line| line.parse::<u64>().ok())
        .map(|e| MessageId::new(e))
        .collect();

    msg_ids
}

async fn generate_embed(
    members: Vec<Member>,
    messages: Vec<Message>,
    ctx: &Context,
) -> CreateEmbed {
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
            increment_streak(&mut member).await;
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
            println!("reset for {:?}", member.name);
            reset_streak(&mut member).await;
            naughty_list.push(member.clone());
        }
    }

    let author = CreateEmbedAuthor::new("amD")
        .url("https://github.com/amfoss/amd")
        .icon_url("https://cdn.discordapp.com/avatars/1245352445736128696/da3c6f833b688f5afa875c9df5d86f91.webp?size=160");

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

    if naughty_list.is_empty() {
        description.push_str("# Missed Updates\nEveryone sent their update yesterday!\n");
        return CreateEmbed::default()
            .title(format!("Status Update Report - {}", today))
            .url("https://www.youtube.com/watch?v=epnuvyNj0FM")
            .description(description)
            .image("https://media1.tenor.com/m/zAHCPvoyjNIAAAAd/yay-kitty.gif")
            .color(serenity::all::Colour::new(0xeab308))
            .timestamp(Timestamp::now())
            .author(author);
    }

    description.push_str("# Missed Updates\n");
    for member in naughty_list {
        println!("{:?}", member.name);
        if member.streak[0].current_streak == 0 {
            description.push_str(&format!("- {} | :x:\n", member.name));
        } else if member.streak[0].current_streak == -1 {
            description.push_str(&format!("- {} | :x::x:\n", member.name));
        } else if member.streak[0].current_streak <= -2 {
            description.push_str(&format!("- {} | :headstone:\n", member.name));
        }
    }

    CreateEmbed::default()
        .title(format!("Status Update Report - {}", today))
        .url("https://www.youtube.com/watch?v=epnuvyNj0FM")
        .description(description)
        .color(serenity::all::Colour::new(0xeab308))
        .timestamp(Timestamp::now())
        .author(author)
}
