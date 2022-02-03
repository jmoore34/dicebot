#[macro_use]
extern crate lazy_static;

mod util;

use core::num;
use std::{
    cmp::{max, min},
    fs::read_to_string, fmt::format,
};

use rand::Rng;
use regex::{Match, Regex};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::channel::Message,
    Client,
};

use crate::util::{roll, format_roll, Roll};

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
        (
            (?:dl?(?P<drop_lowest>\d+))
            |
            (?:dh(?P<drop_highest>\d+))
            |
            (?:kl(?P<keep_lowest>\d+))
            |
            (?:kh?(?P<keep_highest>\d+))
            |
            (?P<disadvantage>d(?:is(?:adv(?:antage)?)?)?)
            |
            (?P<advantage>a(?:dv(?:antage)?)?)
            |
            (?:r(?:eroll)?(?P<reroll>\d+))
            |
            (?P<modifier>
                [+-]
                \d+
            )
        )*
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
            let disadvantage = groups.name("disadvantage").is_some();
            let advantage = groups.name("advantage").is_some();
            let drop_lowest = parse(groups.name("drop_lowest"), 0);
            let drop_highest = parse(groups.name("drop_highest"), 0);
            let keep_lowest = parse(groups.name("keep_lowest"), 0);
            let keep_highest = parse(groups.name("keep_highest"), 0);
            let reroll = parse(groups.name("reroll"), 0);
            let modifier = parse(groups.name("modifier"), 0);
            if dice_size <= 0 // invalid states
                || drop_lowest + drop_highest + keep_lowest + keep_highest > num_dice - 1 // i.e. overlapping drops/keeps
                || drop_lowest != 0 && keep_lowest != 0 // i.e. both dropping and keeping the same rolls
                || drop_highest != 0 && keep_highest != 0
            {
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
                let roll1 = roll(dice_size, reroll);
                let roll2 = roll(dice_size, reroll);

                let roll1_str = format_roll(&roll1, false);
                let roll2_str = format_roll(&roll2, false);

                let full_roll_str = if roll1.value == roll2.value {
                    format!("{roll1_str} / {roll2_str}")
                } else if (advantage && roll1.value > roll2.value) || (disadvantage && roll1.value < roll2.value) {
                    format!("**{roll1_str}** / {roll2_str}")
                } else {
                    format!("{roll1_str} / **{roll2_str}**")
                };

                let sum = modifier
                    + if advantage {
                        max(roll1.value, roll2.value)
                    } else {
                        min(roll1.value, roll2.value)
                    };

                format!("{full_roll_str}{modifier_str} → **{sum}**")
            } else if modifier == 0 && num_dice == 1 {
                let roll = roll(dice_size, reroll);
                format_roll(&roll, false)
            } else {
                let rolls: Vec<Roll> = (1..=num_dice)
                    .map(|_| roll(dice_size, reroll))
                    .collect();
                let sum: i32 = rolls.iter().fold(modifier, |acc, roll| acc + roll.value);
                let roll_str = rolls
                    .iter()
                    .map(|roll| format_roll(roll, false))
                    .collect::<Vec<String>>()
                    .join(" + ");
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
