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

fn claim(svm: &mut litesvm::LiteSVM, program_id: Pubkey, customer: &Keypair, merchant: Pubkey, stats: Pubkey, badge_id: u8) -> Result<(), litesvm::types::FailedTransactionMetadata> {
    let badge_mint = Keypair::new();
    let badge_ata = spl_associated_token_account::get_associated_token_address_with_program_id(&customer.pubkey(), &badge_mint.pubkey(), &spl_token_2022::id());
    let accounts = crosspoint::accounts::ClaimAchievement {
        customer: customer.pubkey(), merchant, customer_stats: stats,
        badge_mint: badge_mint.pubkey(), customer_badge_account: badge_ata,
        token_program: spl_token_2022::id(), associated_token_program: spl_associated_token_account::id(),
        system_program: system_program::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::ClaimAchievement { badge_id }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&customer.pubkey()), &[customer, &badge_mint], svm.latest_blockhash());
    svm.send_transaction(tx).map(|_| ())
}

#[test]
fn claim_achievement_succeeds_once_threshold_met() {
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let customer = new_funded_keypair(&mut svm);
    let (merchant, mint) = register_a_merchant(&mut svm, program_id, &authority);
    let (stats, ata) = enroll(&mut svm, program_id, &customer, merchant, mint);
    purchase(&mut svm, program_id, &authority, customer.pubkey(), merchant, mint, stats, ata, 100); // meets FREQUENT_CUSTOMER_THRESHOLD

    let result = claim(&mut svm, program_id, &customer, merchant, stats, 0);
    assert!(result.is_ok(), "claim_achievement failed: {:?}", result.err());
}

#[test]
fn claim_achievement_rejects_when_not_eligible() {
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let customer = new_funded_keypair(&mut svm);
    let (merchant, mint) = register_a_merchant(&mut svm, program_id, &authority);
    let (stats, ata) = enroll(&mut svm, program_id, &customer, merchant, mint);
    purchase(&mut svm, program_id, &authority, customer.pubkey(), merchant, mint, stats, ata, 10); // below threshold

    let result = claim(&mut svm, program_id, &customer, merchant, stats, 0);
    assert!(result.is_err(), "claim_achievement must reject an ineligible customer");
}

#[test]
fn claim_achievement_rejects_double_claim() {
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let customer = new_funded_keypair(&mut svm);
    let (merchant, mint) = register_a_merchant(&mut svm, program_id, &authority);
    let (stats, ata) = enroll(&mut svm, program_id, &customer, merchant, mint);
    purchase(&mut svm, program_id, &authority, customer.pubkey(), merchant, mint, stats, ata, 100);

    let first = claim(&mut svm, program_id, &customer, merchant, stats, 0);
    assert!(first.is_ok(), "first claim must succeed: {:?}", first.err());

    let second = claim(&mut svm, program_id, &customer, merchant, stats, 0);
    assert!(second.is_err(), "claiming the same badge_id twice must fail");
}

#[test]
fn claim_achievement_rejects_mismatched_customer_stats() {
    // A customer must not be able to claim a badge using someone else's stats PDA, even
    // though customer_stats is a publicly-readable account address.
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let big_spender = new_funded_keypair(&mut svm);
    let freeloader = new_funded_keypair(&mut svm);
    let (merchant, mint) = register_a_merchant(&mut svm, program_id, &authority);
    let (big_spender_stats, big_spender_ata) = enroll(&mut svm, program_id, &big_spender, merchant, mint);
    enroll(&mut svm, program_id, &freeloader, merchant, mint);
    purchase(&mut svm, program_id, &authority, big_spender.pubkey(), merchant, mint, big_spender_stats, big_spender_ata, 100);

    // freeloader signs, but points at big_spender's (eligible) stats PDA.
    let result = claim(&mut svm, program_id, &freeloader, merchant, big_spender_stats, 0);
    assert!(result.is_err(), "claim_achievement must reject a customer_stats account that doesn't belong to the signer");
}

#[test]
fn badge_token_account_rejects_transfer() {
    let (mut svm, program_id) = setup();
    let authority = new_funded_keypair(&mut svm);
    let customer = new_funded_keypair(&mut svm);
    let other_wallet = new_funded_keypair(&mut svm);
    let (merchant, mint) = register_a_merchant(&mut svm, program_id, &authority);
    let (stats, ata) = enroll(&mut svm, program_id, &customer, merchant, mint);
    purchase(&mut svm, program_id, &authority, customer.pubkey(), merchant, mint, stats, ata, 100);

    let badge_mint = Keypair::new();
    let customer_badge_ata = spl_associated_token_account::get_associated_token_address_with_program_id(&customer.pubkey(), &badge_mint.pubkey(), &spl_token_2022::id());
    let other_badge_ata = spl_associated_token_account::get_associated_token_address_with_program_id(&other_wallet.pubkey(), &badge_mint.pubkey(), &spl_token_2022::id());

    let claim_accounts = crosspoint::accounts::ClaimAchievement {
        customer: customer.pubkey(), merchant, customer_stats: stats,
        badge_mint: badge_mint.pubkey(), customer_badge_account: customer_badge_ata,
        token_program: spl_token_2022::id(), associated_token_program: spl_associated_token_account::id(),
        system_program: system_program::id(),
    }.to_account_metas(None);
    let claim_data = crosspoint::instruction::ClaimAchievement { badge_id: 0 }.data();
    let claim_tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts: claim_accounts, data: claim_data }], Some(&customer.pubkey()), &[&customer, &badge_mint], svm.latest_blockhash());
    svm.send_transaction(claim_tx).expect("claim_achievement must succeed");

    // Create the destination ATA, then attempt a standard transfer of the (NonTransferable) badge.
    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &customer.pubkey(), &other_wallet.pubkey(), &badge_mint.pubkey(), &spl_token_2022::id(),
    );
    // Badge mints are created via create_extended_mint, which hardcodes POINTS_DECIMALS (6)
    // for every mint it creates, badges included; decimals here must match or the transfer
    // would fail for the wrong reason (a decimals mismatch) rather than NonTransferable.
    let transfer_ix = spl_token_2022::instruction::transfer_checked(
        &spl_token_2022::id(), &customer_badge_ata, &badge_mint.pubkey(), &other_badge_ata,
        &customer.pubkey(), &[], 1, crosspoint::constants::POINTS_DECIMALS,
    ).unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[create_ata_ix, transfer_ix], Some(&customer.pubkey()), &[&customer], svm.latest_blockhash(),
    );
    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "transferring a NonTransferable badge must be rejected by the token programme itself");
}
