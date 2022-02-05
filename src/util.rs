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

#[derive(Clone, Eq, PartialEq)]
pub struct Roll {
    pub value: i32, // the roll's final value after any rerolling, or initial value if it wasn't rerolled
    pub old_value: Option<i32>, // the dice's original value (only Some() if it was rerolled, else None)
}

impl Ord for Roll {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Roll {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn roll(dice_size: i32, reroll_if_less_than_or_equal_to: i32) -> Roll {
    let roll1 = rand::thread_rng().gen_range(1..=dice_size);
    if roll1 <= reroll_if_less_than_or_equal_to {
        let replacement_roll = rand::thread_rng().gen_range(1..=dice_size);
        Roll {
            value: replacement_roll,
            old_value: Some(roll1),
        }
    } else {
        Roll {
            value: roll1,
            old_value: None,
        }
    }
}

#[derive(Copy, Clone)]
pub enum MarkCondition {
    Highest,
    Lowest,
}
// Given a vector of rolls,
// return a vector of booleans
// with each boolean indicating whether the corresponding roll is 'marked'.
// Marks the highest or lowest num_to_mark rolls, depending on the supplied condition.
// Essentially, this is a min/max function but with the biggest/smallest num_to_mark rolls
// (instead of just the single biggest/smallest)
// PRECONDITION: num_to_mark <= rolls.length - 1
pub fn mark_rolls<T: Ord>(rolls: &Vec<T>, num_to_mark: i32, condition: MarkCondition) -> Vec<bool> {
    // A working vector of the num_to_mark best rolls.
    // "Best" means highest or lowest, corresponding to the condition
    let mut best: Vec<(usize, &T)> = rolls[0..num_to_mark as usize]
        .iter()
        .enumerate()
        .clone()
        .collect();
  
    // Of the rolls in the vector above, which is the worst?
    // E.g. if condition is highest and the best so far are 13, 11, and 15,
    // 11 is the worst of the best
    let mut worst_of_the_best = match condition {
        MarkCondition::Highest => {
            best.iter_mut().min_by(|x,y| x.1.cmp(y.1)).unwrap()
        }
        MarkCondition::Lowest => {
            best.iter_mut().max_by(|x,y| x.1.cmp(y.1)).unwrap()
        }
    };

    // Now iterate through the input array.
    // We skip the first num_to_mark elements because we already put them in the `best` vector
    // (see `best` vector's initialization above)
    for (i, roll) in rolls.iter().enumerate().skip(num_to_mark as usize) {
        // If the roll is better than the worst of the best,
        // we should yoink the worst of the best and replace it with this roll!
        if matches!(condition, MarkCondition::Highest) && roll > worst_of_the_best.1
        || matches!(condition, MarkCondition::Lowest) && roll < worst_of_the_best.1
        {
            // Kick out the old worst of the best:
            // put this new roll where it was.
            *worst_of_the_best = (i, roll);

            // Now, we need to find out which of the best
            // is now the worst.
            // This is not necessarily the new roll.
            // For instance, if the condition was Highest
            // and we had 13, 11, and 15, and we kicked
            // out 11 and replacd it with 14,
            // the new one (14) is not the new worst of the best --
            // 13 is!
            match condition {
                MarkCondition::Highest => {
                    worst_of_the_best = best.iter_mut().min_by(|x,y| x.1.cmp(y.1)).unwrap();
                }
                MarkCondition::Lowest => {
                    worst_of_the_best = best.iter_mut().max_by(|x,y| x.1.cmp(y.1)).unwrap();
                }
            }
        }
    }
    // Now we have a vector of the best rolls.
    // However, we want to return a vector that gives info on 
    // all rolls (e.g. booleans that tell whether they are
    // one of the best ("marked") or not.
    // This is because this format is easier to iterate though
    // since it's ordered by index!
    let result = {
        let mut tmp = vec![false; (*rolls).len()];
        for (index, _roll) in best {
            tmp[index] = true
        }
        tmp
    };

    result
}


#[cfg(test)]
mod tests {
    use crate::util::{MarkCondition, mark_rolls};

    #[test]
    fn highest_3() {
        let input = vec![4,7,8,1,2,6,3,5];
        assert_eq!(mark_rolls(&input, 3, MarkCondition::Highest),
            vec![false, true, true, false, false, true, false, false]);
    }
    #[test]
    fn highest_3b() {
        let input = vec![4,5,8,1,2,6,3,7];
        assert_eq!(mark_rolls(&input, 3, MarkCondition::Highest),
            vec![false, false, true, false, false, true, false, true]);
    }
    #[test]
    fn highest_3c() {
        let input = vec![5,5,1,6,4,6];
        assert_eq!(mark_rolls(&input, 5, MarkCondition::Highest),
            vec![true, true, false, true, true, true]);
    }
    #[test]
    fn highest_3_ascending() {
        let input = vec![1,2,3,4,5,6,7,8];
        assert_eq!(mark_rolls(&input, 3, MarkCondition::Highest),
            vec![false, false, false, false, false, true, true, true]);
    }
    #[test]
    fn highest_3_descending() {
        let input = vec![8,7,6,5,4,3,2,1];
        assert_eq!(mark_rolls(&input, 3, MarkCondition::Highest),
            vec![true,true,true,false,false,false,false,false]);
    }

    #[test]
    fn lowest_3() {
        let input = vec![6,2,3,4,1,6];
        assert_eq!(mark_rolls(&input, 3, MarkCondition::Lowest),
            vec![false, true, true, false, true, false]);
    }
    #[test]
    fn lowest_3b() {
        let input = vec![2,4,1,5,3,4];
        assert_eq!(mark_rolls(&input, 2, MarkCondition::Lowest),
            vec![true, false, true, false, false, false]);
    }
    #[test]
    fn lowest_3c() {
        let input = vec![3,6,1,5];
        assert_eq!(mark_rolls(&input, 2, MarkCondition::Lowest),
            vec![true, false, true, false]);
    }
    #[test]
    fn lowest_3_ascending() {
        let input = vec![1,2,3,4,5,6,7,8];
        assert_eq!(mark_rolls(&input, 3, MarkCondition::Lowest),
            vec![true,true,true,false,false,false,false,false]);

    }
    #[test]
    fn lowest_3_descending() {
        let input = vec![8,7,6,5,4,3,2,1];
        assert_eq!(mark_rolls(&input, 3, MarkCondition::Lowest),
            vec![false, false, false, false, false, true, true, true]);

    }
}