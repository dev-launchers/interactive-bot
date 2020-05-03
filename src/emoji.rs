use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::char;

#[derive(Deserialize, Serialize)]
pub struct LotteryConfig {
    pub has_winner: bool,
    pub active: bool,
    pub jackpot: String,
    pub season: u64,
    pub retry_in_hrs: i64,
    pub config_kv_namespace: String,
    pub data_kv_namespace: String,
}

impl LotteryConfig {
    pub fn kv_key(&self) -> String {
        return "lotteryConfig".to_string();
    }

    pub fn commence(&self) -> Self {
        let jackpot = generate_random_emoji();
        Self {
            has_winner: false,
            active: true,
            jackpot: jackpot.to_string(),
            season: self.season + 1,
            retry_in_hrs: self.retry_in_hrs,
            config_kv_namespace: self.config_kv_namespace.clone(),
            data_kv_namespace: self.data_kv_namespace.clone(),
        }
    }
}

// Generate a random emoji based on v13 of emoji list https://unicode.org/emoji/charts/full-emoji-list.html
// The unicode range is U+1F3F4 to U+1F600
fn generate_random_emoji() -> char {
    use js_sys::Date;

    let ticks = Date::now();
    //convert the number to byte array
    let tick_bytes = transmute(ticks as u128);
    let mut rng = SmallRng::from_seed(tick_bytes);
    // get_range method will hang
    let x: u32 = rng.gen();
    let emoji = char::from_u32(emoji_range(x)).unwrap_or('💡');
    emoji
}

fn emoji_range(x: u32) -> u32 {
    0x1F3F4 + x % (0x1F600 - 0x1F3F4)
}

fn transmute(x: u128) -> [u8; 16] {
    let b1: u8 = ((x >> 120) & 0xffffffff) as u8;
    let b2: u8 = ((x >> 112) & 0xffffffff) as u8;
    let b3: u8 = ((x >> 104) & 0xffffffff) as u8;
    let b4: u8 = ((x >> 96) & 0xffffffff) as u8;

    let b5: u8 = ((x >> 88) & 0xffffffff) as u8;
    let b6: u8 = ((x >> 80) & 0xffffffff) as u8;
    let b7: u8 = ((x >> 72) & 0xffffffff) as u8;
    let b8: u8 = ((x >> 64) & 0xffffffff) as u8;

    let b9: u8 = ((x >> 56) & 0xffffffff) as u8;
    let b10: u8 = ((x >> 48) & 0xffffffff) as u8;
    let b11: u8 = ((x >> 40) & 0xffffffff) as u8;
    let b12: u8 = ((x >> 32) & 0xffffffff) as u8;

    let b13: u8 = ((x >> 24) & 0xffffffff) as u8;
    let b14: u8 = ((x >> 16) & 0xffffffff) as u8;
    let b15: u8 = ((x >> 8) & 0xffffffff) as u8;
    let b16: u8 = (x & 0xffffffff) as u8;

    //Most of the entropy is in the last few bytes and generators are allowed
    //to assume evenly spread entropy in the seed, so spread the bytes around
    [
        b16, b1, b14, b3, b12, b5, b10, b7, b8, b9, b6, b11, b4, b13, b2, b15,
    ]
}
