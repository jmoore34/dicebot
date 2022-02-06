#[macro_use]
extern crate lazy_static;

mod util;
mod eval_dice_expression;

use std::{
    cmp::{max, min},
    env,
    fs::{read_to_string, File}, path::Path,
};

use regex::{Match, Regex};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::channel::Message,
    Client,
};


use crate::util::{format_roll, mark_rolls, roll, MarkCondition, Roll};
use crate::eval_dice_expression::{eval_dice_expression};

#[tokio::main]
async fn main() {
    let version = env!("CARGO_PKG_VERSION");
    println!("DiceBot v{version}");

    let bot_token_filename = "./BOT_TOKEN.txt";

    let bot_token = match env::var("BOT_TOKEN") {
        Ok(token) if token.len() > 50 => Some(token.trim().to_owned()),
        _ => match read_to_string(bot_token_filename) {
            Ok(token) if token.len() > 50 => Some(token.trim().to_owned()),
            _ => None,
        },
    };

    match bot_token {
        Some(token) => {
            println!("Starting server.");
            let mut client = Client::builder(&token)
                .event_handler(Handler)
                .await
                .expect("Error creating client");

            if let Err(why) = client.start().await {
                println!("Client error: {:?}", why);
            }
        }
        None => {
            eprintln!("ERROR: Please provide a bot token, either by setting the BOT_TOKEN environmental variable or by placing it in the {bot_token_filename} file.");
            if !Path::new(bot_token_filename).exists() {
                match File::create(bot_token_filename) {
                    Ok(_) => println!("INFO: Created empty {bot_token_filename} file."),
                    Err(e) => eprintln!("ERROR: Failed to create empty {bot_token_filename}: {e:?}"),
                }
            }
        },
    }
}

struct Handler; // event handler

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        if let Some(result) = eval_dice_expression(&msg.content) {
            if let Err(why) = msg.reply_ping(&ctx.http, result).await {
                println!("Error sending message: {:?}", why);
            }
        }
        
        if msg.content == "69" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "nice").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}
