use litesvm::LiteSVM;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

// Shared LiteSVM setup. LiteSVM::new() already bundles Token, Token-2022, and
// Associated Token Account as builtins (see with_spl_programs() in litesvm's
// own source) — only crosspoint's own compiled programme needs adding.
pub fn setup() -> (LiteSVM, Pubkey) {
    let mut svm = LiteSVM::new();

    let program_id = crosspoint::id();
    // Cargo runs test binaries with cwd set to the package directory (this crate's own
    // manifest folder), not the workspace root — so the shared `target/` two levels up
    // needs an absolute path rather than the plain relative one.
    let so_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/crosspoint.so");
    svm.add_program_from_file(program_id, so_path)
        .expect("load crosspoint.so — run `cargo build-sbf --manifest-path programs/crosspoint/Cargo.toml` first");

    (svm, program_id)
}

pub fn fund(svm: &mut LiteSVM, pubkey: &Pubkey, lamports: u64) {
    svm.airdrop(pubkey, lamports).unwrap();
}

pub fn new_funded_keypair(svm: &mut LiteSVM) -> Keypair {
    let kp = Keypair::new();
    fund(svm, &kp.pubkey(), 10_000_000_000);
    kp
}
