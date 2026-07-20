use crate::constants::LANE_SEED;
use crate::errors::CrossPointError;
use crate::state::{Merchant, TradeLane};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CloseLane<'info> {
    #[account(
        constraint = authority.key() == merchant_a.authority || authority.key() == merchant_b.authority
            @ CrossPointError::UnauthorizedMerchantAuthority
    )]
    pub authority: Signer<'info>,

    pub merchant_a: Account<'info, Merchant>,
    pub merchant_b: Account<'info, Merchant>,

    #[account(
        mut,
        seeds = [LANE_SEED, merchant_a.key().as_ref(), merchant_b.key().as_ref()],
        bump = lane.bump
    )]
    pub lane: Account<'info, TradeLane>,
}

pub fn handler(ctx: Context<CloseLane>) -> Result<()> {
    let lane = &mut ctx.accounts.lane;
    require!(lane.active, CrossPointError::LaneNotActive);
    lane.active = false;
    Ok(())
}
