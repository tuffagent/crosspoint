use crate::constants::STATS_SEED;
use crate::state::{CustomerStats, Merchant};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct EnrollCustomer<'info> {
    #[account(mut)]
    pub customer: Signer<'info>,

    pub merchant: Account<'info, Merchant>,

    #[account(address = merchant.points_mint)]
    pub points_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = customer,
        space = 8 + CustomerStats::INIT_SPACE,
        seeds = [STATS_SEED, customer.key().as_ref(), merchant.key().as_ref()],
        bump
    )]
    pub customer_stats: Account<'info, CustomerStats>,

    #[account(
        init,
        payer = customer,
        associated_token::mint = points_mint,
        associated_token::authority = customer,
        associated_token::token_program = token_program
    )]
    pub customer_points_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<EnrollCustomer>) -> Result<()> {
    let stats = &mut ctx.accounts.customer_stats;
    stats.customer = ctx.accounts.customer.key();
    stats.merchant = ctx.accounts.merchant.key();
    stats.lifetime_earned = 0;
    stats.lifetime_redeemed = 0;
    stats.swap_count = 0;
    stats.achievements_minted = 0;
    stats.bump = ctx.bumps.customer_stats;
    Ok(())
}
