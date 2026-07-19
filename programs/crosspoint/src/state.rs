use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Merchant {
    pub authority: Pubkey,
    pub points_mint: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct TradeLane {
    pub merchant_a: Pubkey,
    pub merchant_b: Pubkey,
    // Fixed-point rate, scaled by RATE_SCALE: how many of B's points one A point buys.
    pub rate_a_to_b: u64,
    pub rate_b_to_a: u64,
    pub active: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct CustomerStats {
    pub customer: Pubkey,
    pub merchant: Pubkey,
    pub lifetime_earned: u64,
    pub lifetime_redeemed: u64,
    pub swap_count: u64,
    // Bitflags, one bit per badge id in constants.rs.
    pub achievements_minted: u8,
    pub bump: u8,
}
