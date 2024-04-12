use std::time::Instant;

use poly_invariant_proof::{
    keccak::Digest as KeccakDigest,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    Withdrawal,
};
use reth_primitives::address;
use sp1_sdk::{SP1Prover, SP1Stdin, SP1Verifier};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {
    // Generate proof.
    let mut stdin = SP1Stdin::new();

    let new_withdrawals = vec![
        Withdrawal::new(
            0,
            0.into(),
            address!("a8da6bf26964af9d7eed9e03e53415d37aa96045"),
            1.into(),
            address!("b8da6bf26964af9d7eed9e03e53415d37aa96045"),
            42_u64.try_into().unwrap(),
            Vec::new(),
        ),
        Withdrawal::new(
            1,
            1.into(),
            address!("c8da6bf26964af9d7eed9e03e53415d37aa96045"),
            0.into(),
            address!("d8da6bf26964af9d7eed9e03e53415d37aa96045"),
            101_u64.try_into().unwrap(),
            Vec::new(),
        ),
    ];
    let initial_exit_tree: LocalExitTree<Keccak256Hasher> =
        LocalExitTree::from_leaves([[0_u8; 32], [1_u8; 32], [2_u8; 32]].into_iter());
    stdin.write(&initial_exit_tree);
    stdin.write(&initial_exit_tree.get_root());
    stdin.write(&new_withdrawals);

    let now = Instant::now();
    let mut proof = SP1Prover::prove(ELF, stdin).expect("proving failed");
    let prover_time = now.elapsed();

    // Read output.
    let _initial_tree_root: KeccakDigest = proof.public_values.read();
    let output_root: KeccakDigest = proof.public_values.read();
    let aggregate_deposits_digest: KeccakDigest = proof.public_values.read();
    println!("new local exit root: {:?}", output_root);
    println!("aggregate deposits digest: {:?}", aggregate_deposits_digest);

    // Verify proof.
    let now = Instant::now();
    SP1Verifier::verify(ELF, &proof).expect("verification failed");
    let verifier_time = now.elapsed();

    println!("successfully generated and verified proof for the program!");
    println!("Prover time: {}ms", prover_time.as_millis());
    println!("Verifier time: {}ms", verifier_time.as_millis());
}
