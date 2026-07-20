use crate::constants::LANE_SEED;
use crate::errors::CrossPointError;
use crate::state::{Merchant, TradeLane};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AcceptLane<'info> {
    #[account(
        constraint = authority.key() == merchant_b.authority @ CrossPointError::UnauthorizedLaneCounterparty
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

pub fn handler(ctx: Context<AcceptLane>) -> Result<()> {
    let lane = &mut ctx.accounts.lane;
    require!(!lane.active, CrossPointError::LaneAlreadyActive);
    lane.active = true;
    Ok(())
}
