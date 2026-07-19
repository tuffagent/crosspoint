// Fixed decimals for every points and badge mint, so swap maths never has to normalise between mints.
pub const POINTS_DECIMALS: u8 = 6;

// Fixed-point scale for TradeLane exchange rates (rate of 1_000_000 == 1:1).
pub const RATE_SCALE: u64 = 1_000_000;

// Achievement thresholds, in raw points (not UI amount) at a single merchant.
pub const FREQUENT_CUSTOMER_THRESHOLD: u64 = 100;
pub const LOYAL_PATRON_THRESHOLD: u64 = 500;

// Achievement bit flags stored in CustomerStats.achievements_minted.
pub const BADGE_FREQUENT_CUSTOMER: u8 = 0;
pub const BADGE_LOYAL_PATRON: u8 = 1;
pub const BADGE_CROSS_MERCHANT_TRADER: u8 = 2;

pub const MERCHANT_SEED: &[u8] = b"merchant";
pub const LANE_SEED: &[u8] = b"lane";
pub const STATS_SEED: &[u8] = b"stats";
pub const BADGE_SEED: &[u8] = b"badge";
