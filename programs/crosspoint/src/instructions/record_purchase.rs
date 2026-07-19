use crate::constants::MERCHANT_SEED;
use crate::errors::CrossPointError;
use crate::state::{CustomerStats, Merchant};
use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct RecordPurchase<'info> {
    #[account(
        constraint = authority.key() == merchant.authority @ CrossPointError::UnauthorizedMerchantAuthority
    )]
    pub authority: Signer<'info>,

    #[account(
        seeds = [MERCHANT_SEED, merchant.authority.as_ref()],
        bump = merchant.bump
    )]
    pub merchant: Account<'info, Merchant>,

    #[account(mut, address = merchant.points_mint)]
    pub points_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub customer_stats: Account<'info, CustomerStats>,

    #[account(mut)]
    pub customer_points_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<RecordPurchase>, amount: u64) -> Result<()> {
    let merchant = &ctx.accounts.merchant;
    let seeds: &[&[u8]] = &[MERCHANT_SEED, merchant.authority.as_ref(), &[merchant.bump]];

    token_2022::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_2022::MintTo {
                mint: ctx.accounts.points_mint.to_account_info(),
                to: ctx.accounts.customer_points_account.to_account_info(),
                authority: merchant.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    let stats = &mut ctx.accounts.customer_stats;
    stats.lifetime_earned = stats
        .lifetime_earned
        .checked_add(amount)
        .ok_or(CrossPointError::MathOverflow)?;

    Ok(())
}
