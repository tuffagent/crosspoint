mod common;

use anchor_lang::{InstructionData, ToAccountMetas};
use common::{new_funded_keypair, setup};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer, system_program,
    transaction::Transaction,
};

#[test]
fn register_merchant_creates_pda_and_mint() {
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let points_mint = Keypair::new();

    let (merchant_pda, _bump) = Pubkey::find_program_address(
        &[b"merchant", authority.pubkey().as_ref()],
        &program_id,
    );

    let accounts = crosspoint::accounts::RegisterMerchant {
        authority: authority.pubkey(),
        merchant: merchant_pda,
        points_mint: points_mint.pubkey(),
        token_program: spl_token_2022::id(),
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = crosspoint::instruction::RegisterMerchant {
        name: "Acme Coffee".to_string(),
        symbol: "ACME".to_string(),
        uri: "".to_string(),
    }
    .data();

    let ix = Instruction { program_id, accounts, data };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority, &points_mint],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "register_merchant failed: {:?}", result.err());

    let merchant_account = svm.get_account(&merchant_pda).expect("merchant PDA must exist");
    assert!(!merchant_account.data.is_empty());
}

#[test]
fn register_merchant_rejects_duplicate_registration() {
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let points_mint_1 = Keypair::new();
    let points_mint_2 = Keypair::new();

    let (merchant_pda, _bump) = Pubkey::find_program_address(
        &[b"merchant", authority.pubkey().as_ref()],
        &program_id,
    );

    let build_tx = |mint: &Keypair, blockhash| {
        let accounts = crosspoint::accounts::RegisterMerchant {
            authority: authority.pubkey(),
            merchant: merchant_pda,
            points_mint: mint.pubkey(),
            token_program: spl_token_2022::id(),
            system_program: system_program::id(),
        }
        .to_account_metas(None);
        let data = crosspoint::instruction::RegisterMerchant {
            name: "Acme Coffee".to_string(),
            symbol: "ACME".to_string(),
            uri: "".to_string(),
        }
        .data();
        Transaction::new_signed_with_payer(
            &[Instruction { program_id, accounts, data }],
            Some(&authority.pubkey()),
            &[&authority, mint],
            blockhash,
        )
    };

    let first = svm.send_transaction(build_tx(&points_mint_1, svm.latest_blockhash()));
    assert!(first.is_ok());

    let second = svm.send_transaction(build_tx(&points_mint_2, svm.latest_blockhash()));
    assert!(second.is_err(), "second registration for the same authority must fail");
}
