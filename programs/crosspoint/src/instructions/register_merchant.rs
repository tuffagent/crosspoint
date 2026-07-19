use crate::constants::MERCHANT_SEED;
use crate::state::Merchant;
use crate::token_2022_helpers::{create_extended_mint, CreateExtendedMintParams};
use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;

#[derive(Accounts)]
pub struct RegisterMerchant<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + Merchant::INIT_SPACE,
        seeds = [MERCHANT_SEED, authority.key().as_ref()],
        bump
    )]
    pub merchant: Account<'info, Merchant>,

    /// CHECK: created manually in the handler body via create_extended_mint.
    #[account(mut)]
    pub points_mint: Signer<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterMerchant>, name: String, symbol: String, uri: String) -> Result<()> {
    let merchant_bump = ctx.bumps.merchant;
    let authority_key = ctx.accounts.authority.key();
    // The Merchant PDA is this mint's authority, so it must sign the TokenMetadataInitialize
    // CPI itself (that instruction requires mint_authority as a signer) — invoke_signed with
    // its own seeds, exactly as record_purchase/swap_points later sign mint_to with this PDA.
    let merchant_seeds: &[&[u8]] = &[MERCHANT_SEED, authority_key.as_ref(), &[merchant_bump]];

    create_extended_mint(CreateExtendedMintParams {
        mint: ctx.accounts.points_mint.to_account_info(),
        payer: ctx.accounts.authority.to_account_info(),
        mint_authority: ctx.accounts.merchant.key(),
        mint_authority_seeds: merchant_seeds,
        mint_authority_account: ctx.accounts.merchant.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        name,
        symbol,
        uri,
        non_transferable: false,
    })?;

    let merchant = &mut ctx.accounts.merchant;
    merchant.authority = authority_key;
    merchant.points_mint = ctx.accounts.points_mint.key();
    merchant.bump = merchant_bump;

    Ok(())
}
