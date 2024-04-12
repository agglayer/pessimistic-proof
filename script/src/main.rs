use std::time::Instant;

use poly_invariant_proof::{
    keccak::Digest as KeccakDigest,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    test_utils::{parse_json_file, DepositEventData},
    Withdrawal,
};
use sp1_sdk::{SP1Prover, SP1Stdin, SP1Verifier};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
const WITHDRAWALS_JSON_FILE_PATH: &str = "src/data/withdrawals.json";

fn main() {
    // Generate proof.
    let mut stdin = SP1Stdin::new();

    let withdrawals_batch: Vec<Withdrawal> = {
        let deposit_event_data: Vec<DepositEventData> = parse_json_file(WITHDRAWALS_JSON_FILE_PATH);

        deposit_event_data.into_iter().map(Into::into).collect()
    };

    let initial_exit_tree: LocalExitTree<Keccak256Hasher> =
        LocalExitTree::from_leaves([[0_u8; 32], [1_u8; 32], [2_u8; 32]].into_iter());
    stdin.write(&initial_exit_tree);
    stdin.write(&initial_exit_tree.get_root());
    stdin.write(&withdrawals_batch);

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
