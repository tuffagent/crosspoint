use crate::constants::{LANE_SEED, MERCHANT_SEED};
use crate::errors::CrossPointError;
use crate::math::convert_amount;
use crate::state::{CustomerStats, Merchant, TradeLane};
use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

// The lane PDA is seeded by the sorted (min, max) merchant pair regardless of which
// direction the customer is swapping in, so either merchant may be passed as
// merchant_from/merchant_to and the same lane account resolves both ways.
#[derive(Accounts)]
pub struct SwapPoints<'info> {
    pub customer: Signer<'info>,

    pub merchant_from: Account<'info, Merchant>,
    pub merchant_to: Account<'info, Merchant>,

    #[account(
        seeds = [
            LANE_SEED,
            std::cmp::min(merchant_from.key(), merchant_to.key()).as_ref(),
            std::cmp::max(merchant_from.key(), merchant_to.key()).as_ref(),
        ],
        bump = lane.bump,
        constraint = lane.active @ CrossPointError::LaneNotActive
    )]
    pub lane: Account<'info, TradeLane>,

    #[account(mut, address = merchant_from.points_mint)]
    pub points_mint_from: InterfaceAccount<'info, Mint>,
    #[account(mut, address = merchant_to.points_mint)]
    pub points_mint_to: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = points_mint_from,
        associated_token::authority = customer,
        associated_token::token_program = token_program
    )]
    pub customer_points_account_from: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = points_mint_to,
        associated_token::authority = customer,
        associated_token::token_program = token_program
    )]
    pub customer_points_account_to: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = customer_stats_to.customer == customer.key() @ CrossPointError::InvalidCustomerStats,
        constraint = customer_stats_to.merchant == merchant_to.key() @ CrossPointError::InvalidCustomerStats
    )]
    pub customer_stats_to: Account<'info, CustomerStats>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<SwapPoints>, amount: u64) -> Result<()> {
    let lane = &ctx.accounts.lane;
    let merchant_from_key = ctx.accounts.merchant_from.key();
    // lane.merchant_a/merchant_b is the sorted-order pair fixed at propose_lane time;
    // pick the rate that matches whichever side merchant_from actually is.
    let rate = if merchant_from_key == lane.merchant_a {
        lane.rate_a_to_b
    } else {
        lane.rate_b_to_a
    };
    let converted = convert_amount(amount, rate)?;
    // A zero (or rounded-to-zero) converted amount would still let a customer bump
    // swap_count and claim the Cross-Merchant Trader badge without moving any real value.
    require!(converted > 0, CrossPointError::SwapAmountTooSmall);

    token_2022::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_2022::Burn {
                mint: ctx.accounts.points_mint_from.to_account_info(),
                from: ctx.accounts.customer_points_account_from.to_account_info(),
                authority: ctx.accounts.customer.to_account_info(),
            },
        ),
        amount,
    )?;

    let merchant_to = &ctx.accounts.merchant_to;
    let seeds: &[&[u8]] = &[MERCHANT_SEED, merchant_to.authority.as_ref(), &[merchant_to.bump]];

    token_2022::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_2022::MintTo {
                mint: ctx.accounts.points_mint_to.to_account_info(),
                to: ctx.accounts.customer_points_account_to.to_account_info(),
                authority: merchant_to.to_account_info(),
            },
            &[seeds],
        ),
        converted,
    )?;

    let stats_to = &mut ctx.accounts.customer_stats_to;
    stats_to.swap_count = stats_to
        .swap_count
        .checked_add(1)
        .ok_or(CrossPointError::MathOverflow)?;

    Ok(())
}
