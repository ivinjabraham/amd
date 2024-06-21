use chrono::Timelike;
use reqwest::blocking::Client as HttpClient;
use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};
use serde::{Deserialize, Serialize};
use reqwest::Error as ReqwestError;


#[derive(Debug, Deserialize)]
struct Student {
    active_time: String,
    last_seen: String,
    login_time: String,
    name: String,
    rollNo: String,
}

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "$amdctl" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "amFOSS Daemon is up and running!").await {
                error!("ERROR: Could not send message: {:?}.", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is online!", ready.user.name);

        schedule_messages(ctx).await;
    }
}

async fn schedule_messages(ctx: Context) {
    let ctx = std::sync::Arc::new(ctx);

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

    loop {
        interval.tick().await;

        let now_utc = chrono::Utc::now();
        let now_kolkata = now_utc.with_timezone(&chrono_tz::Asia::Kolkata);

        // Check if it's time to send messages
    if (now_kolkata.hour() == 17 && now_kolkata.minute() == 30)
        || (now_kolkata.hour() == 18 && now_kolkata.minute() == 0)
        || (now_kolkata.hour() == 19 && now_kolkata.minute() == 0) {

        let (absentees, late) = get_stragglers(&ctx).await;

        let date_str = now_kolkata.format("%d %B %Y").to_string();

        let mut message = format!(
            "# Presense Report - {}\n",
            date_str
        );

        if absentees.len() > 0 {
            message.push_str(&format!("\n## Absent\n"));
            for (index, name) in absentees.iter().enumerate() {
                message.push_str(&format!("{}. {}\n", index + 1, name));
            }
        }


        if late.len() > 0 {
            message.push_str(&format!("\n## Late\n"));
            for (index, name) in late.iter().enumerate() {
                message.push_str(&format!("{}. {}\n", index + 1, name));
            }
        }

        let channel_id = serenity::model::id::ChannelId::new(1252600949164474391);
        if let Err(why) = channel_id.say(&ctx.http, &message).await {
            println!("Error sending message: {:?}", why);
        }
        }
    }
}

async fn get_stragglers(ctx: &std::sync::Arc<serenity::client::Context>) -> (Vec<String>, Vec<String>) {

    let mut absentees = Vec::new();
    let mut late = Vec::new();

    if let Ok(students) = fetch_students().await {
        let now_kolkata = chrono::Utc::now().with_timezone(&chrono_tz::Asia::Kolkata);

        for student in students {
            if student.active_time == "Absent" {
                absentees.push(student.name.clone());
                continue;
            }

            if let Ok(login_time) = chrono::NaiveTime::parse_from_str(&student.login_time, "%H:%M") {
                if login_time > chrono::NaiveTime::from_hms_opt(18, 0, 0).expect("ERROR: Invalid hour, minute or second.") {
                    late.push(student.name.clone());
                }
            } else {
                error!("Error parsing login_time for student: {}", student.name);
            }

            if let Ok(last_seen_time) = chrono::NaiveTime::parse_from_str(&student.last_seen, "%H:%M") {
                let kokl = now_kolkata.time();

                let duration_since_last_seen = kokl.signed_duration_since(last_seen_time);
                let thirty_minutes = chrono::Duration::minutes(30);

                if duration_since_last_seen >= thirty_minutes {
                    absentees.push(student.name.clone());
                }
            } else {
                error!("Error parsing last_seen time for student: {}", student.name);
            }
        }
    }
    (absentees, late)
}

async fn fetch_students() -> Result<Vec<Student>, ReqwestError> {

    let url = "https://labtrack.pythonanywhere.com/current_day";
    let response = reqwest::get(url).await?; // Perform the async GET request
    let students: Vec<Student> = response.json().await?; // Deserialize JSON response
    Ok(students)
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("ERROR: Could not create client.");

    Ok(client.into())
}
