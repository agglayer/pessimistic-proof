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

const INITIAL_LEAF_COUNT: u32 = 1853;

fn main() {
    // Generate proof.
    let mut stdin = SP1Stdin::new();

    let withdrawals_batch: Vec<Withdrawal> = {
        let deposit_event_data: Vec<DepositEventData> = parse_json_file(WITHDRAWALS_JSON_FILE_PATH);

        deposit_event_data.into_iter().map(Into::into).collect()
    };

    let initial_exit_tree: LocalExitTree<Keccak256Hasher> = LocalExitTree::from_parts(
        INITIAL_LEAF_COUNT,
        [
            hex::decode("4a3c0e05a537700590e5cfa29654e7db5b36fbe85b24e7f34bdec7ed2b194aa6")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("167e9d479ed70cdd2918875dd368edacc1b900085a2db71832a951ac7df31e10")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("480549a7a72ab13cb9dd7a1c48f3b2749be3f3a7dd440f16125a1aa5cbf07991")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("81b8a2cf7a80538dee49ae721a87655b080523d37cdad80c6a002a33e91c96cb")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("5003a15ab43bbf7e8a86fe84c7af7a515e8086e53308b4321ac83560e44cd17b")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("02c16029dec2ad77fb3f45ade9b12be2a191dc5bde71e15c5e873695b06eebb2")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("9779f2ddec81f886c42d4813cd3fe44a8e5d077df11dab2d96d8e52e575ad196")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("ff709923054a0745097aa2bd8b74f3434c2ef34ba4245af36efbb7792c719012")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("47ea61b79f448e3d692755fdd7ea1242148f1736e2ec44910ed34397f093364d")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("96f8e65b2aaa2500a40c5f8e72886cbe47248bda77d76d89666e47509649fdba")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("50f7e8cc2d5e5e9f6ce5e5d0352fff94f6569449620e6e6a693b3dfb9d44e683")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .try_into()
                .unwrap(),
        ],
    );
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

    assert_eq!(
        <[u8; 32]>::try_from(
            hex::decode("bd03ab620225bd2dbe77791aced3c995e1d1a4ba3685a72117d4dc3253f57658")
                .unwrap()
        )
        .unwrap(),
        output_root
    );

    // Verify proof.
    let now = Instant::now();
    SP1Verifier::verify(ELF, &proof).expect("verification failed");
    let verifier_time = now.elapsed();

    println!("successfully generated and verified proof for the program!");
    println!("Prover time: {}ms", prover_time.as_millis());
    println!("Verifier time: {}ms", verifier_time.as_millis());
}
