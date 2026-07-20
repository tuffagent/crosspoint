use crate::errors::CrossPointError;
use crate::state::{CustomerStats, Merchant};
use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct RedeemPoints<'info> {
    pub customer: Signer<'info>,

    pub merchant: Account<'info, Merchant>,

    #[account(mut, address = merchant.points_mint)]
    pub points_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = customer_stats.customer == customer.key() @ CrossPointError::InvalidCustomerStats,
        constraint = customer_stats.merchant == merchant.key() @ CrossPointError::InvalidCustomerStats
    )]
    pub customer_stats: Account<'info, CustomerStats>,

    #[account(
        mut,
        associated_token::mint = points_mint,
        associated_token::authority = customer,
        associated_token::token_program = token_program
    )]
    pub customer_points_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<RedeemPoints>, amount: u64) -> Result<()> {
    token_2022::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_2022::Burn {
                mint: ctx.accounts.points_mint.to_account_info(),
                from: ctx.accounts.customer_points_account.to_account_info(),
                authority: ctx.accounts.customer.to_account_info(),
            },
        ),
        amount,
    )?;

    let stats = &mut ctx.accounts.customer_stats;
    stats.lifetime_redeemed = stats
        .lifetime_redeemed
        .checked_add(amount)
        .ok_or(CrossPointError::MathOverflow)?;

    Ok(())
}
