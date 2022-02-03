#[macro_use]
extern crate lazy_static;

mod util;

use std::{cmp::{max, min}, fs::read_to_string};

use rand::Rng;
use regex::{Match, Regex};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::channel::Message,
    Client,
};

use crate::util::get_circled_number;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let bot_token = read_to_string("./BOT_TOKEN").expect("Failed to read file BOT_TOKEN");

    let mut client = Client::builder(&bot_token)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

struct Handler; // event handler

// This function is pulled out from the Handler struct
// because code completion doesn't work in #[async_trait] structs
#[inline]
async fn on_message(ctx: Context, msg: Message) {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?xi) # case insensitive & allow comments
        ^
        (?:
            (?P<num_dice>\d+)?
            d
            (?P<dice_size>\d+)
        )?
        (?P<disadvantage1>d(?:is)?)?
        (?P<advantage1>a(?:dv(?:antage)?)?)?
        (?P<modifier>
            [+-]
            \d+
        )?
        (?P<disadvantage2>d(?:is)?)?
        (?P<advantage2>a(?:dv(?:antage)?)?)?
        $
        "
        )
        .unwrap();
    }

    fn parse(group_match: Option<Match>, default: i32) -> i32 {
        match group_match {
            Some(group_match) => group_match.as_str().parse::<i32>().unwrap_or(default),
            None => default,
        }
    }

    if msg.author.bot {
        return;
    }

    if let Some(groups) = RE.captures(&msg.content.replace(" ", "")) {
        if groups.name("dice_size").is_some() || groups.name("modifier").is_some() {
            let mut num_dice: i32 = parse(groups.name("num_dice"), 1).clamp(1, 100);
            let dice_size = parse(groups.name("dice_size"), 20);
            let disadvantage =
                groups.name("disadvantage1").is_some() || groups.name("disadvantage2").is_some();
            let advantage = !disadvantage
                && (groups.name("advantage1").is_some() || groups.name("advantage2").is_some());
            let modifier = parse(groups.name("modifier"), 0);
            if dice_size <= 0 {
                return;
            }
            if advantage || disadvantage {
                num_dice = 1;
            }
            let modifier_abs = modifier.abs();
            let modifier_str = if modifier > 0 {
                format!(" + {modifier}")
            } else if modifier < 0 {
                format!(" – {modifier_abs}")
            } else {
                "".into()
            };
            let advantage_str = if disadvantage {
                " with disadvantage"
            } else if advantage {
                " with advantage"
            } else {
                ""
            };

            let normalized = format!("{num_dice}d{dice_size}{modifier_str}{advantage_str}");
            
            let result = if advantage || disadvantage {
                let roll1 = rand::thread_rng().gen_range(1..=dice_size);
                let roll2 = rand::thread_rng().gen_range(1..=dice_size);

                let roll1_str = get_circled_number(roll1);
                let roll2_str = get_circled_number(roll2);

                let full_roll_str = if roll1 == roll2 {
                    format!("{roll1_str} / {roll2_str}")
                } else if (advantage && roll1 > roll2) || (disadvantage && roll1 < roll2) {
                    format!("**{roll1_str}** / {roll2_str}")
                } else {
                    format!("{roll1_str} / **{roll2_str}**")
                };

                let sum = modifier + if advantage {
                    max(roll1, roll2)
                } else {
                    min(roll1, roll2)
                };

                format!("{full_roll_str}{modifier_str} → **{sum}**")

            } else if modifier == 0 && num_dice == 1 {
                let roll = rand::thread_rng().gen_range(1..=dice_size);
                get_circled_number(roll)
            } else {
                let rolls: Vec<i32> = (1..=num_dice).map(|_| rand::thread_rng().gen_range(1..=dice_size)).collect();
                let sum: i32 = rolls.iter().sum::<i32>() + modifier;
                let roll_str = rolls.iter().map(|roll| get_circled_number(*roll)).collect::<Vec<String>>().join(" + ");
                format!("{roll_str}{modifier_str} → **{sum}**")
            };

            let complete_message = format!("Rolling {normalized}:\n{result}");

            if let Err(why) = msg.reply_ping(&ctx.http, complete_message).await {
                println!("Error sending message: {:?}", why);
            }

            return;
        }
    }
    if msg.content == "69" {
        if let Err(why) = msg.channel_id.say(&ctx.http, "nice").await {
            println!("Error sending message: {:?}", why);
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        on_message(ctx, msg).await;
    }
}
