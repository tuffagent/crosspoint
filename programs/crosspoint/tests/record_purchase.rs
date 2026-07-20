mod common;

use anchor_lang::{InstructionData, ToAccountMetas};
use common::{new_funded_keypair, setup};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_program, transaction::Transaction,
};

fn register_a_merchant(svm: &mut litesvm::LiteSVM, program_id: Pubkey, authority: &Keypair) -> (Pubkey, Pubkey) {
    let points_mint = Keypair::new();
    let (merchant_pda, _) = Pubkey::find_program_address(&[b"merchant", authority.pubkey().as_ref()], &program_id);
    let accounts = crosspoint::accounts::RegisterMerchant {
        authority: authority.pubkey(), merchant: merchant_pda, points_mint: points_mint.pubkey(),
        token_program: spl_token_2022::id(), system_program: system_program::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::RegisterMerchant {
        name: "Acme Coffee".into(), symbol: "ACME".into(), uri: "".into(),
    }.data();
    let tx = Transaction::new_signed_with_payer(
        &[Instruction { program_id, accounts, data }],
        Some(&authority.pubkey()), &[authority, &points_mint], svm.latest_blockhash(),
    );
    svm.send_transaction(tx).expect("register_merchant must succeed");
    (merchant_pda, points_mint.pubkey())
}

fn enroll(svm: &mut litesvm::LiteSVM, program_id: Pubkey, customer: &Keypair, merchant_pda: Pubkey, points_mint: Pubkey) -> (Pubkey, Pubkey) {
    let (stats_pda, _) = Pubkey::find_program_address(&[b"stats", customer.pubkey().as_ref(), merchant_pda.as_ref()], &program_id);
    let customer_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &customer.pubkey(), &points_mint, &spl_token_2022::id(),
    );
    let accounts = crosspoint::accounts::EnrollCustomer {
        customer: customer.pubkey(), merchant: merchant_pda, points_mint,
        customer_stats: stats_pda, customer_points_account: customer_ata,
        token_program: spl_token_2022::id(), associated_token_program: spl_associated_token_account::id(),
        system_program: system_program::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::EnrollCustomer {}.data();
    let tx = Transaction::new_signed_with_payer(
        &[Instruction { program_id, accounts, data }], Some(&customer.pubkey()), &[customer], svm.latest_blockhash(),
    );
    svm.send_transaction(tx).expect("enroll_customer must succeed");
    (stats_pda, customer_ata)
}

fn token_balance(svm: &litesvm::LiteSVM, ata: Pubkey) -> u64 {
    use spl_token_2022::extension::StateWithExtensions;
    use spl_token_2022::state::Account as TokenAccountState;
    let account = svm.get_account(&ata).expect("token account must exist");
    StateWithExtensions::<TokenAccountState>::unpack(&account.data)
        .expect("must unpack as a Token-2022 account")
        .base
        .amount
}

fn customer_stats(svm: &litesvm::LiteSVM, stats: Pubkey) -> crosspoint::state::CustomerStats {
    use anchor_lang::AccountDeserialize;
    let account = svm.get_account(&stats).expect("stats account must exist");
    crosspoint::state::CustomerStats::try_deserialize(&mut account.data.as_slice()).expect("must deserialize as CustomerStats")
}

#[test]
fn record_purchase_mints_points_and_updates_stats() {
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let customer = new_funded_keypair(&mut svm);
    let (merchant_pda, points_mint) = register_a_merchant(&mut svm, program_id, &authority);
    let (stats_pda, customer_ata) = enroll(&mut svm, program_id, &customer, merchant_pda, points_mint);

    let accounts = crosspoint::accounts::RecordPurchase {
        authority: authority.pubkey(), merchant: merchant_pda, customer: customer.pubkey(), points_mint,
        customer_stats: stats_pda, customer_points_account: customer_ata,
        token_program: spl_token_2022::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::RecordPurchase { amount: 50 }.data();
    let tx = Transaction::new_signed_with_payer(
        &[Instruction { program_id, accounts, data }], Some(&authority.pubkey()), &[&authority], svm.latest_blockhash(),
    );
    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "record_purchase failed: {:?}", result.err());
    // is_ok() alone wouldn't catch a handler that mints the wrong amount or updates the
    // wrong stats field, so assert the actual post-purchase state.
    assert_eq!(token_balance(&svm, customer_ata), 50, "50 points must actually be minted");
    assert_eq!(customer_stats(&svm, stats_pda).lifetime_earned, 50, "lifetime_earned must be incremented by the purchase amount");
}

#[test]
fn record_purchase_rejects_mismatched_customer_stats() {
    // A merchant must not be able to credit one customer's purchase onto a different
    // customer's stats/ATA, the customer field must genuinely bind both accounts.
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let real_customer = new_funded_keypair(&mut svm);
    let other_customer = new_funded_keypair(&mut svm);
    let (merchant_pda, points_mint) = register_a_merchant(&mut svm, program_id, &authority);
    let (_real_stats, _real_ata) = enroll(&mut svm, program_id, &real_customer, merchant_pda, points_mint);
    let (other_stats, other_ata) = enroll(&mut svm, program_id, &other_customer, merchant_pda, points_mint);

    // Claim this purchase is for real_customer, but supply other_customer's stats/ATA.
    let accounts = crosspoint::accounts::RecordPurchase {
        authority: authority.pubkey(), merchant: merchant_pda, customer: real_customer.pubkey(), points_mint,
        customer_stats: other_stats, customer_points_account: other_ata,
        token_program: spl_token_2022::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::RecordPurchase { amount: 50 }.data();
    let tx = Transaction::new_signed_with_payer(
        &[Instruction { program_id, accounts, data }], Some(&authority.pubkey()), &[&authority], svm.latest_blockhash(),
    );
    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "record_purchase must reject a customer_stats/ATA pair that doesn't match the declared customer");
}

#[test]
fn record_purchase_fails_without_prior_enrollment() {
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let customer = new_funded_keypair(&mut svm);
    let (merchant_pda, points_mint) = register_a_merchant(&mut svm, program_id, &authority);

    let (stats_pda, _) = Pubkey::find_program_address(&[b"stats", customer.pubkey().as_ref(), merchant_pda.as_ref()], &program_id);
    let customer_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &customer.pubkey(), &points_mint, &spl_token_2022::id(),
    );

    let accounts = crosspoint::accounts::RecordPurchase {
        authority: authority.pubkey(), merchant: merchant_pda, customer: customer.pubkey(), points_mint,
        customer_stats: stats_pda, customer_points_account: customer_ata,
        token_program: spl_token_2022::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::RecordPurchase { amount: 50 }.data();
    let tx = Transaction::new_signed_with_payer(
        &[Instruction { program_id, accounts, data }], Some(&authority.pubkey()), &[&authority], svm.latest_blockhash(),
    );
    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "record_purchase must fail when the customer never enrolled");
}
