use crate::constants::MERCHANT_SEED;
use crate::errors::CrossPointError;
use crate::math::convert_amount;
use crate::state::{CustomerStats, Merchant, TradeLane};
use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct SwapPoints<'info> {
    pub customer: Signer<'info>,

    pub merchant_a: Account<'info, Merchant>,
    pub merchant_b: Account<'info, Merchant>,

    #[account(
        seeds = [b"lane", merchant_a.key().as_ref(), merchant_b.key().as_ref()],
        bump = lane.bump,
        constraint = lane.active @ CrossPointError::LaneNotActive
    )]
    pub lane: Account<'info, TradeLane>,

    #[account(mut, address = merchant_a.points_mint)]
    pub points_mint_a: InterfaceAccount<'info, Mint>,
    #[account(mut, address = merchant_b.points_mint)]
    pub points_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = points_mint_a,
        associated_token::authority = customer,
        associated_token::token_program = token_program
    )]
    pub customer_points_account_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = points_mint_b,
        associated_token::authority = customer,
        associated_token::token_program = token_program
    )]
    pub customer_points_account_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = customer_stats_b.customer == customer.key() @ CrossPointError::InvalidCustomerStats,
        constraint = customer_stats_b.merchant == merchant_b.key() @ CrossPointError::InvalidCustomerStats
    )]
    pub customer_stats_b: Account<'info, CustomerStats>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<SwapPoints>, amount: u64) -> Result<()> {
    let converted = convert_amount(amount, ctx.accounts.lane.rate_a_to_b)?;

    token_2022::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_2022::Burn {
                mint: ctx.accounts.points_mint_a.to_account_info(),
                from: ctx.accounts.customer_points_account_a.to_account_info(),
                authority: ctx.accounts.customer.to_account_info(),
            },
        ),
        amount,
    )?;

    let merchant_b = &ctx.accounts.merchant_b;
    let seeds: &[&[u8]] = &[MERCHANT_SEED, merchant_b.authority.as_ref(), &[merchant_b.bump]];

    token_2022::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_2022::MintTo {
                mint: ctx.accounts.points_mint_b.to_account_info(),
                to: ctx.accounts.customer_points_account_b.to_account_info(),
                authority: merchant_b.to_account_info(),
            },
            &[seeds],
        ),
        converted,
    )?;

    let stats_b = &mut ctx.accounts.customer_stats_b;
    stats_b.swap_count = stats_b
        .swap_count
        .checked_add(1)
        .ok_or(CrossPointError::MathOverflow)?;

    Ok(())
}
