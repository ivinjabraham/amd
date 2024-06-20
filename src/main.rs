use chrono::Timelike;
use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};

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
        if now_kolkata.hour() == 17 && now_kolkata.minute() == 30 {
            send_message(&ctx, "Message at 5:30 PM").await;
        } else if now_kolkata.hour() == 18 && now_kolkata.minute() == 0 {
            send_message(&ctx, "Message at 6:00 PM").await;
        } else if now_kolkata.hour() == 19 && now_kolkata.minute() == 0 {
            send_message(&ctx, "Message at 7:00 PM").await;
        }
    }
}

async fn send_message(ctx: &std::sync::Arc<serenity::client::Context>, content: &str) {
    let channel_id = serenity::model::id::ChannelId::new(1252600949164474391);
    if let Err(e) = channel_id.say(&ctx.http, content).await {
        error!("Error sending message: {:?}", e);
    }
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
