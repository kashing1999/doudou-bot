use doudoubot::scraper::{Product, Website};
use doudoubot::utils::say_message;

use dotenv::dotenv;
use std::env;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::offset::Local;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::{Activity, Ready};
use serenity::model::id::{ChannelId, GuildId};
use serenity::prelude::*;
use tokio::sync::mpsc;

struct Handler {
    is_loop_running: AtomicBool,
    database: sqlx::Pool<sqlx::MySql>,
    channel_id: u64,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.as_str().starts_with("!remove") {
            let db = self.database.clone();
            let mut m = String::from("Hmm, I didn't quite understand that");
            if let Some((_, u)) = msg.content.rsplit_once(' ') {
                if let Ok(result) = sqlx::query!("DELETE FROM products WHERE url = ?", u)
                    .execute(&db)
                    .await
                {
                    if result.rows_affected() == 0 {
                        m = format!("{u} was not in the database");
                    } else {
                        m = format!("Okay! Removing {u}");
                    }
                } else {
                    m = format!("Problem removing {u} from the database");
                }
            }
            if let Err(why) = msg.channel_id.say(&ctx.http, m).await {
                eprintln!("Error sending message: {:?}", why);
            }
        } else if msg.content.as_str().starts_with("!zaloradd") {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Todo").await {
                eprintln!("Error sending message: {:?}", why);
            }
        } else if msg.content.as_str().starts_with("!zalorm") {
            let db = self.database.clone();
            let mut m = String::from("Hmm, I didn't quite understand that");
            if let Some((_, u)) = msg.content.rsplit_once(' ') {
                if let Ok(result) = sqlx::query!("DELETE FROM zalora WHERE url = ?", u)
                    .execute(&db)
                    .await
                {
                    if result.rows_affected() == 0 {
                        m = format!("{u} was not in the database");
                    } else {
                        m = format!("Okay! Removing {u}");
                    }
                } else {
                    m = format!("Problem removing {u} from the database");
                }
            }
            if let Err(why) = msg.channel_id.say(&ctx.http, m).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
        match msg.content.as_str() {
            "!ping" => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                    eprintln!("Error sending message: {:?}", why);
                }
            }
            "!system" => {
                log_system_load(ctx, self.channel_id).await;
            }
            "!doggo" => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "im cute doggo hehe").await {
                    eprintln!("Error sending message: {:?}", why);
                }
            }
            "!treats" => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "AWOOOOO WHERE").await {
                    eprintln!("Error sending message: {:?}", why);
                }
            }
            "!vendors" => {
                let vendors = vec!["Just a test"];
                let message = format!("Currently supporting {}", vendors.join(", "));
                if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
                    eprintln!("Error sending message: {:?}", why);
                }
            }
            "!help" | "!command" | "!commands" => {
                let message = ChannelId(self.channel_id)
                    .send_message(&ctx, |m| {
                        m.embed(|e| {
                            e.title("Help")
                                .field("!remove", "!remove https://product_to_remove.com", false)
                                .field("!system", "Shows system load", false)
                                .field("!vendors", "Shows supported vendors", false)
                        })
                    })
                    .await;
                if let Err(why) = message {
                    eprintln!("Error sending message: {:?}", why);
                };
            }
            _ => {}
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Cache built successfully!");

        let ctx = Arc::new(ctx);

        if !self.is_loop_running.load(Ordering::Relaxed) {
            // We have to clone the Arc, as it gets moved into the new thread.
            let ctx1 = Arc::clone(&ctx);
            let db = self.database.clone();
            let (tx, mut rx) = mpsc::channel::<Product>(32);
            let channel_id = self.channel_id;

            tokio::spawn(async move {
                while let Some(product) = rx.recv().await {
                    // "SELECT" will return to "entry" the rowid of the todo rows where the user_Id column = user_id.
                    let entry = sqlx::query!(
                        "SELECT id, vendor, url, time FROM products WHERE url = ?",
                        &product.url,
                    )
                    .fetch_one(&db) // < Just one data will be sent to entry
                    .await;

                    match entry {
                        Ok(_) => {}
                        Err(_) => {
                            sqlx::query!(
                                "INSERT INTO products (vendor, url) VALUES (?, ?)",
                                product.vendor,
                                product.url,
                            )
                            .execute(&db)
                            .await
                            .unwrap();

                            let message = format!(
                                "@here Woof! {} just dropped a new product!\n{}",
                                product.vendor, product.url
                            );
                            let ctx2 = ctx1.clone();
                            say_message(ctx2, &message, channel_id).await;
                            println!("New product found! {}", product.url);
                        }
                    };
                }
            });

            let ctx2 = Arc::clone(&ctx);
            tokio::spawn(async move {
                let websites = [Arc::new(Website::new(
                    String::from("Just an Example"),
                    String::from("https://does.not/exist"),
                    String::from("scripts/example.sh"),
                    String::from(r"[Ss]earch term"),
                ))];
                loop {
                    for website in &websites {
                        let website = Arc::clone(website);
                        let tx = tx.clone();
                        let ctx = Arc::clone(&ctx2);
                        tokio::spawn(async move {
                            match website.query_website().await {
                                Ok(products) => {
                                    for p in products {
                                        tx.send(p).await.unwrap();
                                    }
                                }
                                Err(why) => {
                                    say_message(ctx, &why, channel_id).await;
                                }
                            }
                        });
                    }
                    tokio::time::sleep(Duration::from_secs(45)).await;
                }
            });

            // timer
            let ctx4 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    set_status_to_current_time(Arc::clone(&ctx4)).await;
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });

            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

async fn log_system_load(ctx: Context, channel_id: u64) {
    let cpu_load = sys_info::loadavg().unwrap();
    let mem_use = sys_info::mem_info().unwrap();

    let message = ChannelId(channel_id)
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("System Resource Load")
                    .field(
                        "CPU Load Average",
                        format!("{:.2}%", cpu_load.one * 10.0),
                        false,
                    )
                    .field(
                        "Memory Usage",
                        format!(
                            "{:.2} MB Free out of {:.2} MB",
                            mem_use.free as f32 / 1000.0,
                            mem_use.total as f32 / 1000.0
                        ),
                        false,
                    )
            })
        })
        .await;
    if let Err(why) = message {
        eprintln!("Error sending message: {:?}", why);
    };
}

async fn set_status_to_current_time(ctx: Arc<Context>) {
    let current_time = Local::now();
    let formatted_time = current_time.to_rfc2822();

    ctx.set_activity(Activity::competing(&format!(
        "the best bot competition. Last online {formatted_time}"
    )))
    .await;
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let channel_id = env::var("DISCORD_CHANNEL_ID")
        .expect("Expected a token in the environment")
        .parse::<u64>()
        .expect("Channel id has to be an int");

    // Initiate a connection to the database file, creating the file if required.
    let database = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").expect("Expected databse url in environment"))
        .await
        .expect("Couldn't connect to database");

    // Run migrations, which updates the database's schema to the latest version.
    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Couldn't run database migrations");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
            database,
            channel_id,
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}
