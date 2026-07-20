use crate::constants::LANE_SEED;
use crate::errors::CrossPointError;
use crate::state::{Merchant, TradeLane};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ProposeLane<'info> {
    #[account(
        mut,
        constraint = authority.key() == merchant_a.authority @ CrossPointError::UnauthorizedMerchantAuthority
    )]
    pub authority: Signer<'info>,

    pub merchant_a: Account<'info, Merchant>,
    pub merchant_b: Account<'info, Merchant>,

    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + TradeLane::INIT_SPACE,
        seeds = [LANE_SEED, merchant_a.key().as_ref(), merchant_b.key().as_ref()],
        bump
    )]
    pub lane: Account<'info, TradeLane>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ProposeLane>, rate_a_to_b: u64, rate_b_to_a: u64) -> Result<()> {
    require!(
        ctx.accounts.merchant_a.key() != ctx.accounts.merchant_b.key(),
        CrossPointError::SelfLaneNotAllowed
    );
    require!(
        ctx.accounts.merchant_a.key() < ctx.accounts.merchant_b.key(),
        CrossPointError::MerchantsNotSorted
    );
    require!(rate_a_to_b > 0 && rate_b_to_a > 0, CrossPointError::InvalidRate);

    let lane = &mut ctx.accounts.lane;
    require!(!lane.active, CrossPointError::LaneAlreadyActive);

    lane.merchant_a = ctx.accounts.merchant_a.key();
    lane.merchant_b = ctx.accounts.merchant_b.key();
    lane.rate_a_to_b = rate_a_to_b;
    lane.rate_b_to_a = rate_b_to_a;
    lane.active = false;
    lane.bump = ctx.bumps.lane;

    Ok(())
}
