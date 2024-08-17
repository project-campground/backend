const S32_CHAR: &str = "234567abcdefghijklmnopqrstuvwxyz";

pub fn is_s32(s: &str) -> bool {
    // Check if the string decodes into valid s32
    let mut i = 0.0;
    for c in s.chars() {
        let pos = S32_CHAR.find(c);
        if pos == None {
            return false
        }
        i = i * 32.0 + pos.unwrap() as f64;
    }
    true
}

pub fn s32encode(i: f64) -> String {
    let mut s = "".to_owned();
    let mut i = i;
    while i > 0 as f64 {
        let c = i % 32.0;
        i = (i / 32.0).floor();
        s = S32_CHAR.chars().nth(c as usize).unwrap().to_string() + &s;
    }
    s.to_string()
}

pub fn s32decode(s: String) -> f64 {
    let mut i = 0.0;
    for c in s.chars() {
        i = i * 32.0 + S32_CHAR.find(c).unwrap() as f64;
    }
    i
}