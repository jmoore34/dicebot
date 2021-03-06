use std::cmp::{max, min};

use indoc::indoc;
use regex::{Match, Regex};

use crate::util::{format_roll, mark_rolls, roll, MarkCondition, Roll};

pub fn eval_dice_expression(expression: &str) -> Option<String> {
    lazy_static! {
        // (indoc! removes leading whitespace at compile time)
        static ref RE: Regex = Regex::new(indoc! {r"
            (?xi) # case insensitive
            ^
            (?:(?P<repeat1>\d+)\*)?
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
                ((?:\*|rep(?:eat)?)(?P<repeat2>\d+))
                |
                (?:r(?:eroll)?(?P<reroll>\d+))
                |
                (?P<modifier>
                    [+-]
                    \d+
                )
            )*
            $
        "}).unwrap();
    }

    fn parse(group_match: Option<Match>, default: i32) -> i32 {
        match group_match {
            Some(group_match) => group_match.as_str().parse::<i32>().unwrap_or(default),
            None => default,
        }
    }
    fn parse_option(group_match: Option<Match>, max_valid_value: i32) -> Option<i32> {
        match group_match {
            Some(group_match) => match group_match.as_str().parse::<i32>() {
                Ok(num) if (1..=max_valid_value).contains(&num) => Some(num),
                _ => None,
            },
            None => None,
        }
    }

    if let Some(groups) = RE.captures(&expression.replace(" ", "")) {
        if groups.name("dice_size").is_some() || groups.name("modifier").is_some() {
            let mut num_dice: i32 = parse(groups.name("num_dice"), 1).clamp(1, 100);
            let dice_size = parse(groups.name("dice_size"), 20);
            let disadvantage = groups.name("disadvantage").is_some();
            let advantage = groups.name("advantage").is_some();
            // can't drop more than num_dice-1 or there's no dice left. similarly keeping any more than num_dice-1 doesn't make sense
            let drop_lowest = parse_option(groups.name("drop_lowest"), num_dice - 1);
            let drop_highest = parse_option(groups.name("drop_highest"), num_dice - 1);
            let keep_lowest = parse_option(groups.name("keep_lowest"), num_dice - 1);
            let keep_highest = parse_option(groups.name("keep_highest"), num_dice - 1);
            let reroll = parse(groups.name("reroll"), 0);
            let modifier = parse(groups.name("modifier"), 0);
            let repeat1 = parse(groups.name("repeat1"), 1); // repeat syntax can be at beginning or end/with other options
            let repeat2 = parse(groups.name("repeat2"), 1);
            let repeat = max(repeat1, repeat2).clamp(1, 20);
            if advantage || disadvantage {
                num_dice = 1;
            }
            let modifier_abs = modifier.abs();
            let modifier_str = if modifier > 0 {
                format!(" + {modifier}")
            } else if modifier < 0 {
                format!(" ??? {modifier_abs}")
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

            enum DropOrKeep {
                Drop,
                Keep,
            }
            let (drop_or_keep_amount, mark_condition, drop_or_keep) =
                if let Some(amount) = drop_lowest {
                    (
                        Some(amount),
                        Some(MarkCondition::Lowest),
                        Some(DropOrKeep::Drop),
                    )
                } else if let Some(amount) = drop_highest {
                    (
                        Some(amount),
                        Some(MarkCondition::Highest),
                        Some(DropOrKeep::Drop),
                    )
                } else if let Some(amount) = keep_lowest {
                    (
                        Some(amount),
                        Some(MarkCondition::Lowest),
                        Some(DropOrKeep::Keep),
                    )
                } else if let Some(amount) = keep_highest {
                    (
                        Some(amount),
                        Some(MarkCondition::Highest),
                        Some(DropOrKeep::Keep),
                    )
                } else {
                    (None, None, None)
                };
            let drop_or_keep_str = {
                let action = match drop_or_keep {
                    Some(DropOrKeep::Drop) => "dropping",
                    Some(DropOrKeep::Keep) => "keeping",
                    None => "",
                };
                let condition = match mark_condition {
                    Some(MarkCondition::Highest) => "highest",
                    Some(MarkCondition::Lowest) => "lowest",
                    None => "",
                };
                match drop_or_keep_amount {
                    Some(amount) if amount == 1 => format!(", {action} {condition} roll"),
                    Some(amount) => format!(", {action} {condition} {amount} rolls"),
                    None => "".to_owned(),
                }
            };

            let reroll_str = if reroll > 0 {
                let nums_string = (1..=reroll)
                    .map(|i| format!("{i}s"))
                    .collect::<Vec<String>>()
                    .join("/");
                format!(", rerolling {nums_string}")
            } else {
                "".to_owned()
            };

            let repeat_str = if repeat > 1 {
                format!(", repeating {repeat} times")
            } else {
                "".to_owned()
            };

            let normalized =
                format!("{num_dice}d{dice_size}{modifier_str}{advantage_str}{reroll_str}{drop_or_keep_str}{repeat_str}");

            struct Result {
                result_str: String,
                sum: i32
            }
            let aggregate_result = (0..repeat)
                .map(|_idx| {
                    if advantage || disadvantage {
                        let roll1 = roll(dice_size, reroll);
                        let roll2 = roll(dice_size, reroll);

                        let roll1_str = format_roll(&roll1, false);
                        let roll2_str = format_roll(&roll2, false);

                        let full_roll_str = if roll1.value == roll2.value {
                            format!("{roll1_str} / {roll2_str}")
                        } else if (advantage && roll1.value > roll2.value)
                            || (disadvantage && roll1.value < roll2.value)
                        {
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

                        Result {
                            result_str: format!("{full_roll_str}{modifier_str} ??? **{sum}**"),
                            sum: sum
                        }
                    } else if modifier == 0 && num_dice == 1 {
                        let roll = roll(dice_size, reroll);
                        Result {
                            result_str: format_roll(&roll, false),
                            sum: roll.value
                        }
                    } else {
                        let rolls: Vec<Roll> =
                            (1..=num_dice).map(|_| roll(dice_size, reroll)).collect();
                        let (roll_str, sum) = if let Some(drop_or_keep_amount) = drop_or_keep_amount
                        {
                            let marked =
                                mark_rolls(&rolls, drop_or_keep_amount, mark_condition.unwrap());
                            let roll_str = rolls
                                .iter()
                                .zip(marked.iter())
                                .map(|(roll, is_marked)| {
                                    format_roll(
                                        roll,
                                        match drop_or_keep.as_ref().unwrap() {
                                            DropOrKeep::Drop => *is_marked,
                                            DropOrKeep::Keep => !*is_marked,
                                        },
                                    )
                                })
                                .collect::<Vec<String>>()
                                .join(" + ");
                            let sum = rolls.iter().zip(marked.iter()).fold(
                                modifier,
                                |acc, (roll, is_marked)| match drop_or_keep.as_ref().unwrap() {
                                    DropOrKeep::Drop => {
                                        if *is_marked {
                                            acc
                                        } else {
                                            acc + roll.value
                                        }
                                    }
                                    DropOrKeep::Keep => {
                                        if *is_marked {
                                            acc + roll.value
                                        } else {
                                            acc
                                        }
                                    }
                                },
                            );
                            (roll_str, sum)
                        } else {
                            let roll_str = rolls
                                .iter()
                                .map(|roll| format_roll(roll, false))
                                .collect::<Vec<String>>()
                                .join(" + ");
                            let sum = rolls.iter().fold(modifier, |acc, roll| acc + roll.value);
                            (roll_str, sum)
                        };
                        Result {
                            result_str: format!("{roll_str}{modifier_str} ??? **{sum}**"),
                            sum: sum
                        }
                    }
                }).reduce(|mut a, b| {
                    // Aggregate the results
                    Result {
                        result_str: {
                            a.result_str.push('\n');
                            a.result_str.push_str(&b.result_str);
                            a.result_str
                        },
                        sum: a.sum + b.sum
                    }
                }).unwrap();

            // Show the grand total of all the repeated rolls, if applicable
            let total_str = if repeat > 1 {
                let total: i32 = aggregate_result.sum;
                format!("\nTotal: **{total}**")
            } else {
                "".to_owned()
            };

            let result_str = aggregate_result.result_str;
            Some(format!("Rolling {normalized}:\n{result_str}{total_str}"))
        } else {
            None
        }
    } else {
        None
    }
}
