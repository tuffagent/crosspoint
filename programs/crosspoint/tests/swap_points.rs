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
    let data = crosspoint::instruction::RegisterMerchant { name: "Acme".into(), symbol: "ACM".into(), uri: "".into() }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&authority.pubkey()), &[authority, &points_mint], svm.latest_blockhash());
    svm.send_transaction(tx).expect("register_merchant must succeed");
    (merchant_pda, points_mint.pubkey())
}

fn sorted_pair(svm: &mut litesvm::LiteSVM, program_id: Pubkey) -> (Keypair, Pubkey, Pubkey, Keypair, Pubkey, Pubkey) {
    let auth1 = new_funded_keypair(svm);
    let auth2 = new_funded_keypair(svm);
    let (m1, mint1) = register_a_merchant(svm, program_id, &auth1);
    let (m2, mint2) = register_a_merchant(svm, program_id, &auth2);
    if m1 < m2 { (auth1, m1, mint1, auth2, m2, mint2) } else { (auth2, m2, mint2, auth1, m1, mint1) }
}

fn lane_pda(program_id: Pubkey, lo: Pubkey, hi: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"lane", lo.as_ref(), hi.as_ref()], &program_id).0
}

fn propose(svm: &mut litesvm::LiteSVM, program_id: Pubkey, auth_lo: &Keypair, m_lo: Pubkey, m_hi: Pubkey, lane: Pubkey, rate: u64) {
    let accounts = crosspoint::accounts::ProposeLane { authority: auth_lo.pubkey(), merchant_a: m_lo, merchant_b: m_hi, lane, system_program: system_program::id() }.to_account_metas(None);
    let data = crosspoint::instruction::ProposeLane { rate_a_to_b: rate, rate_b_to_a: rate }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&auth_lo.pubkey()), &[auth_lo], svm.latest_blockhash());
    svm.send_transaction(tx).expect("propose_lane must succeed");
}

fn accept(svm: &mut litesvm::LiteSVM, program_id: Pubkey, auth_hi: &Keypair, m_lo: Pubkey, m_hi: Pubkey, lane: Pubkey) {
    let accounts = crosspoint::accounts::AcceptLane { authority: auth_hi.pubkey(), merchant_a: m_lo, merchant_b: m_hi, lane }.to_account_metas(None);
    let data = crosspoint::instruction::AcceptLane {}.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&auth_hi.pubkey()), &[auth_hi], svm.latest_blockhash());
    svm.send_transaction(tx).expect("accept_lane must succeed");
}

fn enroll(svm: &mut litesvm::LiteSVM, program_id: Pubkey, customer: &Keypair, merchant: Pubkey, mint: Pubkey) -> (Pubkey, Pubkey) {
    let (stats, _) = Pubkey::find_program_address(&[b"stats", customer.pubkey().as_ref(), merchant.as_ref()], &program_id);
    let ata = spl_associated_token_account::get_associated_token_address_with_program_id(&customer.pubkey(), &mint, &spl_token_2022::id());
    let accounts = crosspoint::accounts::EnrollCustomer {
        customer: customer.pubkey(), merchant, points_mint: mint, customer_stats: stats, customer_points_account: ata,
        token_program: spl_token_2022::id(), associated_token_program: spl_associated_token_account::id(), system_program: system_program::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::EnrollCustomer {}.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&customer.pubkey()), &[customer], svm.latest_blockhash());
    svm.send_transaction(tx).expect("enroll_customer must succeed");
    (stats, ata)
}

fn purchase(svm: &mut litesvm::LiteSVM, program_id: Pubkey, authority: &Keypair, customer: Pubkey, merchant: Pubkey, mint: Pubkey, stats: Pubkey, ata: Pubkey, amount: u64) {
    let accounts = crosspoint::accounts::RecordPurchase {
        authority: authority.pubkey(), merchant, customer, points_mint: mint, customer_stats: stats, customer_points_account: ata,
        token_program: spl_token_2022::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::RecordPurchase { amount }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&authority.pubkey()), &[authority], svm.latest_blockhash());
    svm.send_transaction(tx).expect("record_purchase must succeed");
}

#[test]
fn swap_points_burns_a_and_mints_b_at_rate() {
    let (mut svm, program_id) = setup();
    let (auth_lo, m_lo, mint_lo, auth_hi, m_hi, mint_hi) = sorted_pair(&mut svm, program_id);
    let lane = lane_pda(program_id, m_lo, m_hi);
    propose(&mut svm, program_id, &auth_lo, m_lo, m_hi, lane, 2_000_000); // 1 lo-point = 2 hi-points
    accept(&mut svm, program_id, &auth_hi, m_lo, m_hi, lane);

    let customer = new_funded_keypair(&mut svm);
    let (stats_lo, ata_lo) = enroll(&mut svm, program_id, &customer, m_lo, mint_lo);
    let (stats_hi, ata_hi) = enroll(&mut svm, program_id, &customer, m_hi, mint_hi);
    purchase(&mut svm, program_id, &auth_lo, customer.pubkey(), m_lo, mint_lo, stats_lo, ata_lo, 100);

    let accounts = crosspoint::accounts::SwapPoints {
        customer: customer.pubkey(), merchant_a: m_lo, merchant_b: m_hi, lane,
        points_mint_a: mint_lo, points_mint_b: mint_hi,
        customer_points_account_a: ata_lo, customer_points_account_b: ata_hi,
        customer_stats_b: stats_hi, token_program: spl_token_2022::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::SwapPoints { amount: 50 }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&customer.pubkey()), &[&customer], svm.latest_blockhash());
    assert!(svm.send_transaction(tx).is_ok());
    // 50 lo-points at rate 2_000_000 (2x) => 100 hi-points minted.
}

#[test]
fn swap_points_rejects_inactive_lane() {
    let (mut svm, program_id) = setup();
    let (auth_lo, m_lo, mint_lo, _auth_hi, m_hi, mint_hi) = sorted_pair(&mut svm, program_id);
    let lane = lane_pda(program_id, m_lo, m_hi);
    propose(&mut svm, program_id, &auth_lo, m_lo, m_hi, lane, 2_000_000);
    // Deliberately skip accept() so the lane stays inactive.

    let customer = new_funded_keypair(&mut svm);
    let (stats_lo, ata_lo) = enroll(&mut svm, program_id, &customer, m_lo, mint_lo);
    let (stats_hi, ata_hi) = enroll(&mut svm, program_id, &customer, m_hi, mint_hi);
    purchase(&mut svm, program_id, &auth_lo, customer.pubkey(), m_lo, mint_lo, stats_lo, ata_lo, 100);

    let accounts = crosspoint::accounts::SwapPoints {
        customer: customer.pubkey(), merchant_a: m_lo, merchant_b: m_hi, lane,
        points_mint_a: mint_lo, points_mint_b: mint_hi,
        customer_points_account_a: ata_lo, customer_points_account_b: ata_hi,
        customer_stats_b: stats_hi, token_program: spl_token_2022::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::SwapPoints { amount: 50 }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&customer.pubkey()), &[&customer], svm.latest_blockhash());
    assert!(svm.send_transaction(tx).is_err(), "swap_points must reject an inactive lane");
}

#[test]
fn swap_points_rejects_mismatched_customer_stats_b() {
    // A customer must not be able to redirect this swap's swap_count credit onto a
    // different customer's CustomerStats at merchant B.
    let (mut svm, program_id) = setup();
    let (auth_lo, m_lo, mint_lo, auth_hi, m_hi, mint_hi) = sorted_pair(&mut svm, program_id);
    let lane = lane_pda(program_id, m_lo, m_hi);
    propose(&mut svm, program_id, &auth_lo, m_lo, m_hi, lane, 2_000_000);
    accept(&mut svm, program_id, &auth_hi, m_lo, m_hi, lane);

    let customer = new_funded_keypair(&mut svm);
    let other_customer = new_funded_keypair(&mut svm);
    let (stats_lo, ata_lo) = enroll(&mut svm, program_id, &customer, m_lo, mint_lo);
    let (_stats_hi, ata_hi) = enroll(&mut svm, program_id, &customer, m_hi, mint_hi);
    // A second, unrelated customer also enrolls at merchant B, giving us a real
    // (but wrong) customer_stats_b PDA to substitute in below.
    let (other_stats_hi, _other_ata_hi) = enroll(&mut svm, program_id, &other_customer, m_hi, mint_hi);
    purchase(&mut svm, program_id, &auth_lo, customer.pubkey(), m_lo, mint_lo, stats_lo, ata_lo, 100);

    let accounts = crosspoint::accounts::SwapPoints {
        customer: customer.pubkey(), merchant_a: m_lo, merchant_b: m_hi, lane,
        points_mint_a: mint_lo, points_mint_b: mint_hi,
        customer_points_account_a: ata_lo, customer_points_account_b: ata_hi,
        customer_stats_b: other_stats_hi, token_program: spl_token_2022::id(),
    }.to_account_metas(None);
    let data = crosspoint::instruction::SwapPoints { amount: 50 }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&customer.pubkey()), &[&customer], svm.latest_blockhash());
    assert!(svm.send_transaction(tx).is_err(), "swap_points must reject a customer_stats_b that doesn't belong to the declared customer");
}
