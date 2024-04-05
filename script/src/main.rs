use std::time::Instant;

use poly_invariant_proof::{
    hasher::keccak::{Keccak256Hasher, KeccakDigest},
    local_exit_tree::{withdrawal::Withdrawal, LocalExitTree},
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
            0,
            address!("a8da6bf26964af9d7eed9e03e53415d37aa96045"),
            1,
            address!("b8da6bf26964af9d7eed9e03e53415d37aa96045"),
            42.into(),
            Vec::new(),
        ),
        Withdrawal::new(
            1,
            1,
            address!("c8da6bf26964af9d7eed9e03e53415d37aa96045"),
            0,
            address!("d8da6bf26964af9d7eed9e03e53415d37aa96045"),
            101.into(),
            Vec::new(),
        ),
    ];
    let initial_exit_tree: LocalExitTree<KeccakDigest> =
        LocalExitTree::from_leaves::<Keccak256Hasher>(
            [[0_u8; 32], [1_u8; 32], [2_u8; 32]].into_iter(),
        );
    stdin.write(&initial_exit_tree);
    stdin.write(&initial_exit_tree.get_root::<Keccak256Hasher>());
    stdin.write(&new_withdrawals);

    let now = Instant::now();
    let mut proof = SP1Prover::prove(ELF, stdin).expect("proving failed");
    let prover_time = now.elapsed();

    // Read output.
    let output_root = proof.public_values.read::<KeccakDigest>();
    println!("new local exit root: {:?}", output_root);

    // Verify proof.
    let now = Instant::now();
    SP1Verifier::verify(ELF, &proof).expect("verification failed");
    let verifier_time = now.elapsed();

    println!("successfully generated and verified proof for the program!");
    println!("Prover time: {}ms", prover_time.as_millis());
    println!("Verifier time: {}ms", verifier_time.as_millis());
}
