use blueshift_vault::{Deposit, Withdraw, ID as PROGRAM_ID, SEED};
use litesvm::LiteSVM;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

#[test]
fn deposit_then_withdraw() {
    let mut svm = LiteSVM::new();

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 2 * LAMPORTS_PER_SOL).unwrap();

    let program_bytes = include_bytes!("../target/deploy/blueshift_vault.so");
    let program_id = Pubkey::new_from_array(PROGRAM_ID);

    let (vault_pubkey, bump) = solana_pubkey::Pubkey::try_find_program_address(
        &[SEED, payer.pubkey().as_array()],
        &program_id,
    )
    .unwrap();

    let balance = svm.get_balance(&payer.pubkey()).unwrap();
    assert_eq!(balance, 2 * LAMPORTS_PER_SOL);

    svm.add_program(program_id, program_bytes);

    let mut data = Vec::<u8>::default();
    data.push(*Deposit::DISCRIMINATOR);
    data.push(bump);
    data.extend_from_slice(&LAMPORTS_PER_SOL.to_le_bytes());

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(vault_pubkey, false),
            AccountMeta::new(solana_system_interface::program::ID, false),
        ],
        data,
    };

    let recent_blockhash = svm.latest_blockhash();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    // Simulate tx
    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);

    let deposit_balance = svm.get_balance(&vault_pubkey).unwrap_or_default();
    assert_eq!(LAMPORTS_PER_SOL, deposit_balance);

    let data = vec![*Withdraw::DISCRIMINATOR, bump];
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(vault_pubkey, false),
            AccountMeta::new(solana_system_interface::program::ID, false),
        ],
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    // Simulate tx
    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);

    assert_eq!(svm.get_balance(&vault_pubkey), Some(0));
}
