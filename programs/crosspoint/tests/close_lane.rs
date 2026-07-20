mod common;

use anchor_lang::{InstructionData, ToAccountMetas};
use common::{new_funded_keypair, setup};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_program, transaction::Transaction,
};

fn register_a_merchant(svm: &mut litesvm::LiteSVM, program_id: Pubkey, authority: &Keypair) -> Pubkey {
    let points_mint = Keypair::new();
    let (merchant_pda, _) = Pubkey::find_program_address(&[b"merchant", authority.pubkey().as_ref()], &program_id);
    let accounts = crosspoint::accounts::RegisterMerchant { authority: authority.pubkey(), merchant: merchant_pda, points_mint: points_mint.pubkey(), token_program: spl_token_2022::id(), system_program: system_program::id() }.to_account_metas(None);
    let data = crosspoint::instruction::RegisterMerchant { name: "Acme".into(), symbol: "ACM".into(), uri: "".into() }.data();
    let tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts, data }], Some(&authority.pubkey()), &[authority, &points_mint], svm.latest_blockhash());
    svm.send_transaction(tx).expect("register_merchant must succeed");
    merchant_pda
}

fn sorted_pair(svm: &mut litesvm::LiteSVM, program_id: Pubkey) -> (Keypair, Pubkey, Keypair, Pubkey) {
    let auth1 = new_funded_keypair(svm);
    let auth2 = new_funded_keypair(svm);
    let m1 = register_a_merchant(svm, program_id, &auth1);
    let m2 = register_a_merchant(svm, program_id, &auth2);
    if m1 < m2 { (auth1, m1, auth2, m2) } else { (auth2, m2, auth1, m1) }
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

#[test]
fn close_lane_then_reopen_with_new_rate() {
    let (mut svm, program_id) = setup();
    let (auth_lo, m_lo, auth_hi, m_hi) = sorted_pair(&mut svm, program_id);
    let lane = lane_pda(program_id, m_lo, m_hi);
    propose(&mut svm, program_id, &auth_lo, m_lo, m_hi, lane, 1_000_000);
    accept(&mut svm, program_id, &auth_hi, m_lo, m_hi, lane);

    let close_accounts = crosspoint::accounts::CloseLane { authority: auth_lo.pubkey(), merchant_a: m_lo, merchant_b: m_hi, lane }.to_account_metas(None);
    let close_data = crosspoint::instruction::CloseLane {}.data();
    let close_tx = Transaction::new_signed_with_payer(&[Instruction { program_id, accounts: close_accounts, data: close_data }], Some(&auth_lo.pubkey()), &[&auth_lo], svm.latest_blockhash());
    assert!(svm.send_transaction(close_tx).is_ok());

    // Expire blockhash to prevent AlreadyProcessed dedupe on subsequent propose/accept cycle.
    svm.expire_blockhash();

    // Renegotiate at double the old rate — must succeed now the lane is inactive.
    propose(&mut svm, program_id, &auth_lo, m_lo, m_hi, lane, 2_000_000);
    accept(&mut svm, program_id, &auth_hi, m_lo, m_hi, lane);
}
