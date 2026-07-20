use anchor_client::{Client, Cluster};
use anyhow::Result;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey,
    signature::{Keypair, Signer}, system_program,
};
use std::rc::Rc;

pub fn run_demo(cluster: Cluster, payer: Rc<Keypair>) -> Result<()> {
    let client = Client::new_with_options(cluster.clone(), payer.clone(), CommitmentConfig::confirmed());
    let program = client.program(crosspoint::id())?;
    let rpc = program.rpc();

    let merchant_a_authority = Keypair::new();
    let merchant_b_authority = Keypair::new();
    let customer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();

    for kp in [&merchant_a_authority, &merchant_b_authority, &customer] {
        let sig = rpc.request_airdrop(&kp.pubkey(), 2_000_000_000)?;
        loop {
            if rpc.confirm_transaction(&sig)? { break; }
        }
    }

    println!("Registering merchant A...");
    let (merchant_a, _) = Pubkey::find_program_address(&[b"merchant", merchant_a_authority.pubkey().as_ref()], &crosspoint::id());
    let sig = program
        .request()
        .accounts(crosspoint::accounts::RegisterMerchant {
            authority: merchant_a_authority.pubkey(), merchant: merchant_a, points_mint: mint_a.pubkey(),
            token_program: spl_token_2022::id(), system_program: system_program::id(),
        })
        .args(crosspoint::instruction::RegisterMerchant { name: "Riverside Roasters".into(), symbol: "RVSD".into(), uri: "".into() })
        .signer(&merchant_a_authority)
        .signer(&mint_a)
        .send()?;
    println!("  tx: {sig}");

    println!("Registering merchant B...");
    let (merchant_b, _) = Pubkey::find_program_address(&[b"merchant", merchant_b_authority.pubkey().as_ref()], &crosspoint::id());
    let sig = program
        .request()
        .accounts(crosspoint::accounts::RegisterMerchant {
            authority: merchant_b_authority.pubkey(), merchant: merchant_b, points_mint: mint_b.pubkey(),
            token_program: spl_token_2022::id(), system_program: system_program::id(),
        })
        .args(crosspoint::instruction::RegisterMerchant { name: "Harbourfront Books".into(), symbol: "HRBR".into(), uri: "".into() })
        .signer(&merchant_b_authority)
        .signer(&mint_b)
        .send()?;
    println!("  tx: {sig}");

    let customer_ata_a = spl_associated_token_account::get_associated_token_address_with_program_id(&customer.pubkey(), &mint_a.pubkey(), &spl_token_2022::id());
    let customer_ata_b = spl_associated_token_account::get_associated_token_address_with_program_id(&customer.pubkey(), &mint_b.pubkey(), &spl_token_2022::id());
    let (stats_a, _) = Pubkey::find_program_address(&[b"stats", customer.pubkey().as_ref(), merchant_a.as_ref()], &crosspoint::id());
    let (stats_b, _) = Pubkey::find_program_address(&[b"stats", customer.pubkey().as_ref(), merchant_b.as_ref()], &crosspoint::id());

    println!("Enrolling customer at merchant A...");
    let sig = program
        .request()
        .accounts(crosspoint::accounts::EnrollCustomer {
            customer: customer.pubkey(), merchant: merchant_a, points_mint: mint_a.pubkey(),
            customer_stats: stats_a, customer_points_account: customer_ata_a,
            token_program: spl_token_2022::id(), associated_token_program: spl_associated_token_account::id(),
            system_program: system_program::id(),
        })
        .args(crosspoint::instruction::EnrollCustomer {})
        .signer(&customer)
        .send()?;
    println!("  tx: {sig}");

    println!("Enrolling customer at merchant B...");
    let sig = program
        .request()
        .accounts(crosspoint::accounts::EnrollCustomer {
            customer: customer.pubkey(), merchant: merchant_b, points_mint: mint_b.pubkey(),
            customer_stats: stats_b, customer_points_account: customer_ata_b,
            token_program: spl_token_2022::id(), associated_token_program: spl_associated_token_account::id(),
            system_program: system_program::id(),
        })
        .args(crosspoint::instruction::EnrollCustomer {})
        .signer(&customer)
        .send()?;
    println!("  tx: {sig}");

    println!("Recording a 150-point purchase at merchant A...");
    let sig = program
        .request()
        .accounts(crosspoint::accounts::RecordPurchase {
            authority: merchant_a_authority.pubkey(), merchant: merchant_a, customer: customer.pubkey(),
            points_mint: mint_a.pubkey(), customer_stats: stats_a, customer_points_account: customer_ata_a,
            token_program: spl_token_2022::id(),
        })
        .args(crosspoint::instruction::RecordPurchase { amount: 150 })
        .signer(&merchant_a_authority)
        .send()?;
    println!("  tx: {sig}");

    // propose_lane/accept_lane require merchant_a/merchant_b in strict sorted-pubkey
    // order, which depends on the randomly-generated authority keypairs above, so sort
    // the two merchants here before deriving the lane PDA and proposing.
    let (lane_lo, lane_lo_auth, lane_hi, lane_hi_auth) = if merchant_a < merchant_b {
        (merchant_a, &merchant_a_authority, merchant_b, &merchant_b_authority)
    } else {
        (merchant_b, &merchant_b_authority, merchant_a, &merchant_a_authority)
    };
    let (lane, _) = Pubkey::find_program_address(&[b"lane", lane_lo.as_ref(), lane_hi.as_ref()], &crosspoint::id());

    println!("Proposing a trade lane between the two merchants...");
    let sig = program
        .request()
        .accounts(crosspoint::accounts::ProposeLane {
            authority: lane_lo_auth.pubkey(), merchant_a: lane_lo, merchant_b: lane_hi, lane,
            system_program: system_program::id(),
        })
        .args(crosspoint::instruction::ProposeLane { rate_a_to_b: 500_000, rate_b_to_a: 2_000_000 })
        .signer(lane_lo_auth)
        .send()?;
    println!("  tx: {sig}");

    println!("Accepting the trade lane...");
    let sig = program
        .request()
        .accounts(crosspoint::accounts::AcceptLane { authority: lane_hi_auth.pubkey(), merchant_a: lane_lo, merchant_b: lane_hi, lane })
        .args(crosspoint::instruction::AcceptLane {})
        .signer(lane_hi_auth)
        .send()?;
    println!("  tx: {sig}");

    println!("Swapping 50 of the customer's merchant-A points into merchant-B points...");
    let sig = program
        .request()
        .accounts(crosspoint::accounts::SwapPoints {
            customer: customer.pubkey(), merchant_from: merchant_a, merchant_to: merchant_b, lane,
            points_mint_from: mint_a.pubkey(), points_mint_to: mint_b.pubkey(),
            customer_points_account_from: customer_ata_a, customer_points_account_to: customer_ata_b,
            customer_stats_to: stats_b, token_program: spl_token_2022::id(),
        })
        .args(crosspoint::instruction::SwapPoints { amount: 50 })
        .signer(&customer)
        .send()?;
    println!("  tx: {sig}");

    println!("Claiming the Frequent Customer badge (150 lifetime points earned at merchant A)...");
    let badge_mint = Keypair::new();
    let customer_badge_account = spl_associated_token_account::get_associated_token_address_with_program_id(&customer.pubkey(), &badge_mint.pubkey(), &spl_token_2022::id());
    let sig = program
        .request()
        .accounts(crosspoint::accounts::ClaimAchievement {
            customer: customer.pubkey(), merchant: merchant_a, customer_stats: stats_a,
            badge_mint: badge_mint.pubkey(), customer_badge_account,
            token_program: spl_token_2022::id(), associated_token_program: spl_associated_token_account::id(),
            system_program: system_program::id(),
        })
        .args(crosspoint::instruction::ClaimAchievement { badge_id: 0 }) // BADGE_FREQUENT_CUSTOMER
        .signer(&customer)
        .signer(&badge_mint)
        .send()?;
    println!("  tx: {sig}");

    println!("Redeeming 20 of the customer's remaining merchant-A points...");
    let sig = program
        .request()
        .accounts(crosspoint::accounts::RedeemPoints {
            customer: customer.pubkey(), merchant: merchant_a, points_mint: mint_a.pubkey(),
            customer_stats: stats_a, customer_points_account: customer_ata_a, token_program: spl_token_2022::id(),
        })
        .args(crosspoint::instruction::RedeemPoints { amount: 20 })
        .signer(&customer)
        .send()?;
    println!("  tx: {sig}");

    let bal_a = rpc.get_token_account_balance(&customer_ata_a)?;
    let bal_b = rpc.get_token_account_balance(&customer_ata_b)?;
    println!("Final balances: merchant A points = {}, merchant B points = {}", bal_a.ui_amount_string, bal_b.ui_amount_string);

    println!("Demo complete.");
    Ok(())
}
