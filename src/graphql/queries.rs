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
use crate::graphql::models::{Member, Streak};
use anyhow::{anyhow, Context};

pub async fn fetch_members() -> anyhow::Result<Vec<Member>> {
    let request_url = std::env::var("ROOT_URL").context("ROOT_URL not found in ENV")?;

    let client = reqwest::Client::new();
    let query = r#"
        { 
          members {
            memberId
            name
            discordId
            streak {
              currentStreak
              maxStreak
            }
        }
    }"#;

    let response = client
        .post(request_url)
        .json(&serde_json::json!({"query": query}))
        .send()
        .await
        .context("Failed to successfully post request")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Server responded with an error: {:?}",
            response.status()
        ));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to serialize response")?;

    let members = response_json
        .get("data")
        .and_then(|data| data.get("members"))
        .and_then(|members| members.as_array())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Malformed response: Could not access Members from {}",
                response_json
            )
        })?;

    let members: Vec<Member> = serde_json::from_value(serde_json::Value::Array(members.clone()))
        .context("Failed to parse 'members' into Vec<Member>")?;

    Ok(members)
}

pub async fn increment_streak(member: &mut Member) -> anyhow::Result<()> {
    let request_url = std::env::var("ROOT_URL").context("ROOT_URL was not found in ENV")?;

    let client = reqwest::Client::new();
    let mutation = format!(
        r#"
        mutation {{
            incrementStreak(input: {{ memberId: {} }}) {{
                currentStreak
            }}
        }}"#,
        member.member_id
    );

    let response = client
        .post(request_url)
        .json(&serde_json::json!({"query": mutation}))
        .send()
        .await
        .context("Failed to succesfully post query to Root")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Server responded with an error: {:?}",
            response.status()
        ));
    }

    // Handle the streak vector
    if member.streak.is_empty() {
        // If the streak vector is empty, add a new Streak object with both values set to 1
        member.streak.push(Streak {
            current_streak: 1,
            max_streak: 1,
        });
    } else {
        // Otherwise, increment the current_streak for each Streak and update max_streak if necessary
        for streak in &mut member.streak {
            streak.current_streak += 1;
            if streak.current_streak > streak.max_streak {
                streak.max_streak = streak.current_streak;
            }
        }
    }

    Ok(())
}

pub async fn reset_streak(member: &mut Member) -> anyhow::Result<()> {
    let request_url = std::env::var("ROOT_URL").context("ROOT_URL was not found in the ENV")?;

    let client = reqwest::Client::new();
    let mutation = format!(
        r#"
        mutation {{
            resetStreak(input: {{ memberId: {} }}) {{
                currentStreak
                maxStreak
            }}
        }}"#,
        member.member_id
    );

    let response = client
        .post(&request_url)
        .json(&serde_json::json!({ "query": mutation }))
        .send()
        .await
        .context("Failed to succesfully post query to Root")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Server responded with an error: {:?}",
            response.status()
        ));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse response JSON")?;

    if let Some(data) = response_json
        .get("data")
        .and_then(|data| data.get("resetStreak"))
    {
        let current_streak =
            data.get("currentStreak")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow!("current_streak was parsed as None"))? as i32;
        let max_streak =
            data.get("maxStreak")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow!("max_streak was parsed as None"))? as i32;

        // Update the member's streak vector
        if member.streak.is_empty() {
            // If the streak vector is empty, initialize it with the returned values
            member.streak.push(Streak {
                current_streak,
                max_streak,
            });
        } else {
            // Otherwise, update the first streak entry
            for streak in &mut member.streak {
                streak.current_streak = current_streak;
                streak.max_streak = max_streak;
            }
        }
    } else {
        return Err(anyhow!("Failed to access data from {}", response_json));
    }

    Ok(())
}
