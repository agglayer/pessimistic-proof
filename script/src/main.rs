use std::{collections::HashMap, time::Instant};

use poly_pessimistic_proof::{
    batch::Batch,
    keccak::Digest as KeccakDigest,
    local_balance_tree::{Balance, BalanceTree, Deposit},
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    test_utils::{parse_json_file, DepositEventData},
    NetworkId, TokenInfo, Withdrawal,
};
use reth_primitives::{address, U256};
use sp1_sdk::{ProverClient, SP1Stdin};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
const WITHDRAWALS_JSON_FILE_PATH: &str = "src/data/withdrawals.json";

const INITIAL_LEAF_COUNT: u32 = 1853;

fn make_batch(origin_network: NetworkId) -> Batch {
    let withdrawals: Vec<Withdrawal> = {
        let deposit_event_data: Vec<DepositEventData> = parse_json_file(WITHDRAWALS_JSON_FILE_PATH);

        deposit_event_data.into_iter().map(Into::into).collect()
    };

    let prev_local_exit_tree: LocalExitTree<Keccak256Hasher> = LocalExitTree::from_parts(
        INITIAL_LEAF_COUNT,
        [
            digest_from_hex("4a3c0e05a537700590e5cfa29654e7db5b36fbe85b24e7f34bdec7ed2b194aa6"),
            digest_from_hex("167e9d479ed70cdd2918875dd368edacc1b900085a2db71832a951ac7df31e10"),
            digest_from_hex("480549a7a72ab13cb9dd7a1c48f3b2749be3f3a7dd440f16125a1aa5cbf07991"),
            digest_from_hex("81b8a2cf7a80538dee49ae721a87655b080523d37cdad80c6a002a33e91c96cb"),
            digest_from_hex("5003a15ab43bbf7e8a86fe84c7af7a515e8086e53308b4321ac83560e44cd17b"),
            digest_from_hex("02c16029dec2ad77fb3f45ade9b12be2a191dc5bde71e15c5e873695b06eebb2"),
            digest_from_hex("9779f2ddec81f886c42d4813cd3fe44a8e5d077df11dab2d96d8e52e575ad196"),
            digest_from_hex("ff709923054a0745097aa2bd8b74f3434c2ef34ba4245af36efbb7792c719012"),
            digest_from_hex("47ea61b79f448e3d692755fdd7ea1242148f1736e2ec44910ed34397f093364d"),
            digest_from_hex("96f8e65b2aaa2500a40c5f8e72886cbe47248bda77d76d89666e47509649fdba"),
            digest_from_hex("50f7e8cc2d5e5e9f6ce5e5d0352fff94f6569449620e6e6a693b3dfb9d44e683"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
            digest_from_hex("0000000000000000000000000000000000000000000000000000000000000000"),
        ],
    );

    let prev_local_balance_tree: BalanceTree = {
        let eth = TokenInfo {
            origin_network: origin_network.clone(),
            origin_token_address: address!("0000000000000000000000000000000000000000"),
        };

        let token = TokenInfo {
            origin_network: origin_network.clone(),
            origin_token_address: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
        };

        let infinite_eth = || -> (TokenInfo, Balance) { (eth.clone(), Deposit(U256::MAX).into()) };
        let infinite_token =
            || -> (TokenInfo, Balance) { (token.clone(), Deposit(U256::MAX).into()) };

        BalanceTree::from(vec![infinite_eth(), infinite_token()])
    };

    let prev_local_exit_root = prev_local_exit_tree.get_root();

    Batch {
        origin_network,
        prev_local_exit_tree,
        prev_local_exit_root,
        prev_local_balance_tree,
        withdrawals,
    }
}

fn main() {
    sp1_sdk::utils::setup_logger();

    // Generate proof.
    let mut stdin = SP1Stdin::new();
    let client = ProverClient::new();
    let (proving_key, verifying_key) = client.setup(ELF);

    // Make a single batch from network 0.
    let origin_network: NetworkId = 0.into();
    let batches = vec![make_batch(origin_network)];
    stdin.write(&batches);

    let now = Instant::now();
    let mut proof = client.prove(&proving_key, stdin).expect("proving failed");
    let prover_time = now.elapsed();

    // Read output.
    let new_roots: HashMap<NetworkId, (KeccakDigest, KeccakDigest)> = proof.public_values.read();
    let (exit_root, _balance_root) = new_roots.get(&origin_network).expect("nonexistent network");

    if *exit_root
        == digest_from_hex("bd03ab620225bd2dbe77791aced3c995e1d1a4ba3685a72117d4dc3253f57658")
    {
        println!("Output root is as expected!");
    } else {
        println!("Oops, output root is incorrect");
    }

    // Verify proof.
    let now = Instant::now();
    client
        .verify(&proof, &verifying_key)
        .expect("verification failed");
    let verifier_time = now.elapsed();

    println!("successfully generated and verified proof for the program!");
    println!("Prover time: {}ms", prover_time.as_millis());
    println!("Verifier time: {}ms", verifier_time.as_millis());
}

fn digest_from_hex(hex_digest: &str) -> KeccakDigest {
    hex::decode(hex_digest).unwrap().try_into().unwrap()
}
