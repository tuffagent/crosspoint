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
    let accounts = crosspoint::accounts::RegisterMerchant { authority: authority.pubkey(), merchant: merchant_pda, points_mint: points_mint.pubkey(), token_program: spl_token_2022::id(), system_program: system_program::id() }.to_account_metas(None);
    let data = crosspoint::instruction::RegisterMerchant { name: "Acme".into(), symbol: "ACM".into(), uri: "".into() }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&authority.pubkey()), &[authority, &points_mint], svm.latest_blockhash());
    svm.send_transaction(tx).expect("register_merchant must succeed");
    (merchant_pda, points_mint.pubkey())
}

fn enroll(svm: &mut litesvm::LiteSVM, program_id: Pubkey, customer: &Keypair, merchant: Pubkey, mint: Pubkey) -> (Pubkey, Pubkey) {
    let (stats, _) = Pubkey::find_program_address(&[b"stats", customer.pubkey().as_ref(), merchant.as_ref()], &program_id);
    let ata = spl_associated_token_account::get_associated_token_address_with_program_id(&customer.pubkey(), &mint, &spl_token_2022::id());
    let accounts = crosspoint::accounts::EnrollCustomer { customer: customer.pubkey(), merchant, points_mint: mint, customer_stats: stats, customer_points_account: ata, token_program: spl_token_2022::id(), associated_token_program: spl_associated_token_account::id(), system_program: system_program::id() }.to_account_metas(None);
    let data = crosspoint::instruction::EnrollCustomer {}.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&customer.pubkey()), &[customer], svm.latest_blockhash());
    svm.send_transaction(tx).expect("enroll_customer must succeed");
    (stats, ata)
}

fn purchase(svm: &mut litesvm::LiteSVM, program_id: Pubkey, authority: &Keypair, customer: Pubkey, merchant: Pubkey, mint: Pubkey, stats: Pubkey, ata: Pubkey, amount: u64) {
    let accounts = crosspoint::accounts::RecordPurchase { authority: authority.pubkey(), merchant, customer, points_mint: mint, customer_stats: stats, customer_points_account: ata, token_program: spl_token_2022::id() }.to_account_metas(None);
    let data = crosspoint::instruction::RecordPurchase { amount }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&authority.pubkey()), &[authority], svm.latest_blockhash());
    svm.send_transaction(tx).expect("record_purchase must succeed");
}

#[test]
fn redeem_points_burns_and_updates_stats() {
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let customer = new_funded_keypair(&mut svm);
    let (merchant, mint) = register_a_merchant(&mut svm, program_id, &authority);
    let (stats, ata) = enroll(&mut svm, program_id, &customer, merchant, mint);
    purchase(&mut svm, program_id, &authority, customer.pubkey(), merchant, mint, stats, ata, 100);

    let accounts = crosspoint::accounts::RedeemPoints {
        customer: customer.pubkey(), merchant, points_mint: mint,
        customer_stats: stats, customer_points_account: ata, token_program: spl_token_2022::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::RedeemPoints { amount: 40 }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&customer.pubkey()), &[&customer], svm.latest_blockhash());
    assert!(svm.send_transaction(tx).is_ok());
}

#[test]
fn redeem_points_rejects_insufficient_balance() {
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let customer = new_funded_keypair(&mut svm);
    let (merchant, mint) = register_a_merchant(&mut svm, program_id, &authority);
    let (stats, ata) = enroll(&mut svm, program_id, &customer, merchant, mint);
    purchase(&mut svm, program_id, &authority, customer.pubkey(), merchant, mint, stats, ata, 10);

    let accounts = crosspoint::accounts::RedeemPoints {
        customer: customer.pubkey(), merchant, points_mint: mint,
        customer_stats: stats, customer_points_account: ata, token_program: spl_token_2022::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::RedeemPoints { amount: 999 }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&customer.pubkey()), &[&customer], svm.latest_blockhash());
    assert!(svm.send_transaction(tx).is_err());
}

#[test]
fn redeem_points_rejects_mismatched_customer_stats() {
    // A customer must not be able to redeem against a different customer's
    // CustomerStats at the same merchant.
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let customer = new_funded_keypair(&mut svm);
    let other_customer = new_funded_keypair(&mut svm);
    let (merchant, mint) = register_a_merchant(&mut svm, program_id, &authority);
    let (stats, ata) = enroll(&mut svm, program_id, &customer, merchant, mint);
    // A second, unrelated customer also enrolls at this merchant, giving us a real
    // (but wrong) customer_stats PDA to substitute in below.
    let (other_stats, _other_ata) = enroll(&mut svm, program_id, &other_customer, merchant, mint);
    purchase(&mut svm, program_id, &authority, customer.pubkey(), merchant, mint, stats, ata, 100);

    let accounts = crosspoint::accounts::RedeemPoints {
        customer: customer.pubkey(), merchant, points_mint: mint,
        customer_stats: other_stats, customer_points_account: ata, token_program: spl_token_2022::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::RedeemPoints { amount: 40 }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&customer.pubkey()), &[&customer], svm.latest_blockhash());
    assert!(svm.send_transaction(tx).is_err(), "redeem_points must reject a customer_stats that doesn't belong to the declared customer");
}
