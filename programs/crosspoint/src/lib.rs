use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod math;
pub mod state;
pub mod token_2022_helpers;

use instructions::*;

declare_id!("B2V3qSDVknbsfLL1ZGfcRxjQqnvUNRppF76X1YXEPNLF");

// CrossPoint: cross-merchant loyalty points on Token-2022.
#[program]
pub mod crosspoint {
    use super::*;

    pub fn register_merchant(ctx: Context<RegisterMerchant>, name: String, symbol: String, uri: String) -> Result<()> {
        instructions::register_merchant::handler(ctx, name, symbol, uri)
    }

    pub fn enroll_customer(ctx: Context<EnrollCustomer>) -> Result<()> {
        instructions::enroll_customer::handler(ctx)
    }

    pub fn propose_lane(ctx: Context<ProposeLane>, rate_a_to_b: u64, rate_b_to_a: u64) -> Result<()> {
        instructions::propose_lane::handler(ctx, rate_a_to_b, rate_b_to_a)
    }

    pub fn accept_lane(ctx: Context<AcceptLane>) -> Result<()> {
        instructions::accept_lane::handler(ctx)
    }

    pub fn close_lane(ctx: Context<CloseLane>) -> Result<()> {
        instructions::close_lane::handler(ctx)
    }

    pub fn record_purchase(ctx: Context<RecordPurchase>, amount: u64) -> Result<()> {
        instructions::record_purchase::handler(ctx, amount)
    }

    pub fn redeem_points(ctx: Context<RedeemPoints>, amount: u64) -> Result<()> {
        instructions::redeem_points::handler(ctx, amount)
    }

    pub fn swap_points(ctx: Context<SwapPoints>, amount: u64) -> Result<()> {
        instructions::swap_points::handler(ctx, amount)
    }

    pub fn claim_achievement(ctx: Context<ClaimAchievement>, badge_id: u8) -> Result<()> {
        instructions::claim_achievement::handler(ctx, badge_id)
    }
}
