use crate::constants::{
    BADGE_CROSS_MERCHANT_TRADER, BADGE_FREQUENT_CUSTOMER, BADGE_LOYAL_PATRON,
    FREQUENT_CUSTOMER_THRESHOLD, LOYAL_PATRON_THRESHOLD, MERCHANT_SEED,
};
use crate::errors::CrossPointError;
use crate::state::{CustomerStats, Merchant};
use crate::token_2022_helpers::{create_extended_mint, CreateExtendedMintParams};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::{self, AssociatedToken};
use anchor_spl::token_2022::{self, Token2022};

#[derive(Accounts)]
#[instruction(badge_id: u8)]
pub struct ClaimAchievement<'info> {
    #[account(mut)]
    pub customer: Signer<'info>,

    pub merchant: Account<'info, Merchant>,

    #[account(
        mut,
        has_one = merchant @ CrossPointError::InvalidCustomerStats,
        has_one = customer @ CrossPointError::InvalidCustomerStats
    )]
    pub customer_stats: Account<'info, CustomerStats>,

    /// CHECK: created manually via create_extended_mint; a fresh Keypair signer per claim.
    #[account(mut)]
    pub badge_mint: Signer<'info>,

    // Not an Anchor init + associated_token::mint account: try_accounts runs before the
    // handler, but badge_mint only becomes a real mint inside the handler below.
    /// CHECK: created manually via associated_token::create in the handler, once badge_mint exists.
    #[account(mut)]
    pub customer_badge_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

fn badge_bit(badge_id: u8) -> Result<u8> {
    match badge_id {
        BADGE_FREQUENT_CUSTOMER | BADGE_LOYAL_PATRON | BADGE_CROSS_MERCHANT_TRADER => Ok(badge_id),
        _ => Err(CrossPointError::UnknownBadge.into()),
    }
}

fn badge_metadata(badge_id: u8) -> (&'static str, &'static str) {
    match badge_id {
        BADGE_FREQUENT_CUSTOMER => ("CrossPoint Frequent Customer", "XPFC"),
        BADGE_LOYAL_PATRON => ("CrossPoint Loyal Patron", "XPLP"),
        _ => ("CrossPoint Cross-Merchant Trader", "XPCT"),
    }
}

pub fn handler(ctx: Context<ClaimAchievement>, badge_id: u8) -> Result<()> {
    let bit = badge_bit(badge_id)?;
    let stats = &ctx.accounts.customer_stats;

    let eligible = match bit {
        BADGE_FREQUENT_CUSTOMER => stats.lifetime_earned >= FREQUENT_CUSTOMER_THRESHOLD,
        BADGE_LOYAL_PATRON => stats.lifetime_earned >= LOYAL_PATRON_THRESHOLD,
        _ => stats.swap_count >= 1,
    };
    require!(eligible, CrossPointError::AchievementNotEligible);

    let already_minted = (stats.achievements_minted >> bit) & 1 == 1;
    require!(!already_minted, CrossPointError::AchievementAlreadyMinted);

    let merchant = &ctx.accounts.merchant;
    let seeds: &[&[u8]] = &[MERCHANT_SEED, merchant.authority.as_ref(), &[merchant.bump]];
    let (name, symbol) = badge_metadata(bit);

    create_extended_mint(CreateExtendedMintParams {
        mint: ctx.accounts.badge_mint.to_account_info(),
        payer: ctx.accounts.customer.to_account_info(),
        mint_authority: merchant.key(),
        mint_authority_seeds: seeds,
        mint_authority_account: merchant.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        name: name.to_string(),
        symbol: symbol.to_string(),
        uri: "".to_string(),
        non_transferable: true,
    })?;

    // badge_mint now exists, so the customer's badge ATA can be created against it; the
    // associated-token programme derives and checks this address itself, so a wrong
    // customer_badge_account pubkey is rejected here regardless.
    associated_token::create(CpiContext::new(
        ctx.accounts.associated_token_program.to_account_info(),
        associated_token::Create {
            payer: ctx.accounts.customer.to_account_info(),
            associated_token: ctx.accounts.customer_badge_account.to_account_info(),
            authority: ctx.accounts.customer.to_account_info(),
            mint: ctx.accounts.badge_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    ))?;

    token_2022::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_2022::MintTo {
                mint: ctx.accounts.badge_mint.to_account_info(),
                to: ctx.accounts.customer_badge_account.to_account_info(),
                authority: merchant.to_account_info(),
            },
            &[seeds],
        ),
        1,
    )?;

    ctx.accounts.customer_stats.achievements_minted |= 1 << bit;

    Ok(())
}
