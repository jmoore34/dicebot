lazy_static! {
    static ref CIRCLED_NUMS: Vec<char> =
        "⓪①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳㉑㉒㉓㉔㉕㉖㉗㉘㉙㉚㉛㉜㉝㉞㉟㊱㊲㊳㊴㊵㊶㊷㊸㊹㊺㊻㊼㊽㊾㊿"
            .chars()
            .collect();
}

pub fn get_circled_number(num: i32) -> String {
    if num <= 0 {
        "X".to_string()
    } else if num > 50 {
        format!("({num})")
    } else {
        CIRCLED_NUMS[num as usize].to_string()
    }
}
