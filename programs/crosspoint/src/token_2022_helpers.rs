use crate::errors::CrossPointError;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_2022_extensions::{
    metadata_pointer_initialize, non_transferable_mint_initialize, spl_token_metadata_interface,
    token_metadata_initialize, MetadataPointerInitialize, NonTransferableMintInitialize,
    TokenMetadataInitialize,
};
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::state::Mint as Token2022Mint;

pub struct CreateExtendedMintParams<'a, 'info> {
    pub mint: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub mint_authority: Pubkey,
    // Seeds that let the program invoke_signed as `mint_authority` for the TokenMetadata CPI
    // (e.g. the Merchant PDA's own seeds). Empty for a non-PDA authority.
    pub mint_authority_seeds: &'a [&'a [u8]],
    pub mint_authority_account: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub non_transferable: bool,
}

// Creates a Token-2022 mint with MetadataPointer + on-chain TokenMetadata, optionally also
// NonTransferable, at the fixed CrossPoint decimals. `mint` must be a fresh Keypair-derived
// account that has already signed the transaction (checked by Anchor's `Signer<'info>` on the
// caller's Accounts struct) — this function does the create_account + extension CPIs manually,
// since Anchor's `init` constraint can't add extensions before InitializeMint2.
pub fn create_extended_mint(params: CreateExtendedMintParams) -> Result<()> {
    let CreateExtendedMintParams {
        mint,
        payer,
        mint_authority,
        mint_authority_seeds,
        mint_authority_account,
        token_program,
        system_program,
        name,
        symbol,
        uri,
        non_transferable,
    } = params;

    let mut extensions = vec![ExtensionType::MetadataPointer];
    if non_transferable {
        extensions.push(ExtensionType::NonTransferable);
    }

    let base_len = ExtensionType::try_calculate_account_len::<Token2022Mint>(&extensions)
        .map_err(|_| CrossPointError::MathOverflow)?;
    let rent = Rent::get()?;
    let base_lamports = rent.minimum_balance(base_len);

    // 1. Create the account at the fixed base length (fixed-size extensions only).
    invoke(
        &system_instruction::create_account(
            payer.key,
            mint.key,
            base_lamports,
            base_len as u64,
            token_program.key,
        ),
        &[payer.clone(), mint.clone(), system_program.clone()],
    )?;

    // 2. Initialise fixed-size extensions before InitializeMint2 (Token-2022 requires this order).
    // Self-metadata pattern: the metadata address IS the mint itself.
    metadata_pointer_initialize(
        CpiContext::new(
            token_program.clone(),
            MetadataPointerInitialize {
                token_program_id: token_program.clone(),
                mint: mint.clone(),
            },
        ),
        Some(mint_authority),
        Some(mint.key()),
    )?;

    if non_transferable {
        non_transferable_mint_initialize(CpiContext::new(
            token_program.clone(),
            NonTransferableMintInitialize {
                token_program_id: token_program.clone(),
                mint: mint.clone(),
            },
        ))?;
    }

    // 3. InitializeMint2 — no signer required, just writes decimals/authority into the account.
    invoke(
        &spl_token_2022::instruction::initialize_mint2(
            token_program.key,
            mint.key,
            &mint_authority,
            None,
            crate::constants::POINTS_DECIMALS,
        )
        .map_err(|_| CrossPointError::MathOverflow)?,
        &[mint.clone()],
    )?;

    // 4. Top up rent for the variable-length TokenMetadata payload, then initialise it.
    let metadata = spl_token_metadata_interface::state::TokenMetadata {
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        ..Default::default()
    };
    let extra_len = metadata
        .tlv_size_of()
        .map_err(|_| CrossPointError::MathOverflow)?;
    let extra_lamports = rent
        .minimum_balance(base_len + extra_len)
        .saturating_sub(mint.lamports());
    if extra_lamports > 0 {
        invoke(
            &system_instruction::transfer(payer.key, mint.key, extra_lamports),
            &[payer.clone(), mint.clone(), system_program.clone()],
        )?;
    }

    let metadata_ctx = CpiContext::new(
        token_program.clone(),
        TokenMetadataInitialize {
            token_program_id: token_program.clone(),
            metadata: mint.clone(),
            update_authority: mint_authority_account.clone(),
            mint: mint.clone(),
            mint_authority: mint_authority_account.clone(),
        },
    );
    // Bound to a local so the reference `with_signer` stores outlives this statement
    // (a `&[mint_authority_seeds]` temporary would otherwise drop before `metadata_ctx` is used).
    let signer_seeds_array = [mint_authority_seeds];
    let metadata_ctx = if mint_authority_seeds.is_empty() {
        metadata_ctx
    } else {
        metadata_ctx.with_signer(&signer_seeds_array)
    };
    token_metadata_initialize(metadata_ctx, name, symbol, uri)?;

    Ok(())
}
