use rand::Rng;

lazy_static! {
    static ref CIRCLED_NUMS: Vec<char> =
        "⓪①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳㉑㉒㉓㉔㉕㉖㉗㉘㉙㉚㉛㉜㉝㉞㉟㊱㊲㊳㊴㊵㊶㊷㊸㊹㊺㊻㊼㊽㊾㊿"
            .chars()
            .collect();
}

fn get_circled_number(num: i32) -> String {
    if num <= 0 {
        "X".to_string()
    } else if num > 50 {
        format!("({num})")
    } else {
        CIRCLED_NUMS[num as usize].to_string()
    }
}

pub fn format_roll(roll: &Roll, strikethrough: bool) -> String {
    let value = get_circled_number(roll.value);
    if let Some(old_value) = roll.old_value {
        let old_value = get_circled_number(old_value);
        if strikethrough {
            format!("~~{old_value}{value}~~")
        } else {
            format!("~~{old_value}~~{value}")
        }
    } else {
        if strikethrough {
            format!("~~{value}~~")
        } else {
            value
        }
    }
}

pub struct Roll {
    pub value: i32,  // the roll's final value after any rerolling, or initial value if it wasn't rerolled
    pub old_value: Option<i32> // the dice's original value (only Some() if it was rerolled, else None)
}
pub fn roll(dice_size: i32, reroll_if_less_than_or_equal_to: i32) -> Roll {
    let roll1 = rand::thread_rng().gen_range(1..=dice_size);
    if roll1 <= reroll_if_less_than_or_equal_to {
        let replacement_roll = rand::thread_rng().gen_range(1..=dice_size);
        Roll {
            value: replacement_roll,
            old_value: Some(roll1)
        }
    } else {
        Roll {
            value: roll1,
            old_value: None
        }
    }
}
