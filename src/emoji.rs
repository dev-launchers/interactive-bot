#[derive(Deserialize)]
pub struct LotteryConfig {
    pub has_winner: bool,
    pub active: bool,
    pub jackpot: String,
    season: u64,
    pub retry_in_hrs: i64,
}
