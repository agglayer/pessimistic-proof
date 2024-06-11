use std::collections::HashMap;

use lazy_static::lazy_static;
use poly_pessimistic_proof::{
    certificate::Certificate,
    generate_full_proof,
    local_balance_tree::{Balance, BalanceTree, Deposit},
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    FullProofOutput, NetworkId, ProofError, TokenInfo, Withdrawal,
};
use reth_primitives::{address, U256};
use rstest::{fixture, rstest};

lazy_static! {
    pub static ref USDC: TokenInfo = TokenInfo {
        origin_network: 0.into(),
        origin_token_address: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
    };
    pub static ref ETH: TokenInfo = TokenInfo {
        origin_network: 0.into(),
        origin_token_address: address!("0000000000000000000000000000000000000000"),
    };
}

fn make_tx(_from: u32, to: u32, token: &TokenInfo, amount: u32) -> Withdrawal {
    Withdrawal::new(
        0,
        token.origin_network,
        token.origin_token_address,
        to.into(),
        address!("a8da6bf26964af9d7eed9e03e53415d37aa96045"),
        U256::from(amount),
        Vec::new(),
    )
}

pub struct State {
    pub global_exit_tree: HashMap<NetworkId, LocalExitTree<Keccak256Hasher>>,
    pub global_balance_tree: HashMap<NetworkId, BalanceTree>,
}

pub fn generate_full_proof_with_state(
    initial_state: State,
    certificates: &mut [Certificate],
) -> Result<FullProofOutput, ProofError> {
    // TODO: do not embed the full state in the certificates
    for c in certificates.iter_mut() {
        c.prev_local_balance_tree = initial_state
            .global_balance_tree
            .get(&c.origin_network)
            .expect("non-existent network")
            .clone();

        let exit_tree = initial_state
            .global_exit_tree
            .get(&c.origin_network)
            .expect("non-existent network")
            .clone();

        c.prev_local_exit_root = exit_tree.get_root();
        c.prev_local_exit_tree = exit_tree;
    }

    generate_full_proof(certificates)
}

#[fixture]
fn initial_state() -> State {
    let deposit_eth =
        |v: u32| -> (TokenInfo, Balance) { (ETH.clone(), Deposit(U256::from(v)).into()) };
    let deposit_usdc =
        |v: u32| -> (TokenInfo, Balance) { (USDC.clone(), Deposit(U256::from(v)).into()) };

    let dummy_let = LocalExitTree::from_leaves([[0_u8; 32], [1_u8; 32], [2_u8; 32]].into_iter());
    //let dummy_ler = dummy_let.get_root();

    let initial_0 = BalanceTree::from(vec![deposit_eth(10), deposit_usdc(10)]);
    let initial_1 = BalanceTree::from(vec![deposit_eth(1), deposit_usdc(200)]);

    State {
        global_exit_tree: [(0.into(), dummy_let.clone()), (1.into(), dummy_let.clone())].into(),
        global_balance_tree: [(0.into(), initial_0), (1.into(), initial_1)].into(),
    }
}

#[fixture]
fn state_transition() -> Vec<Certificate> {
    let eth = ETH.clone();
    let usdc = USDC.clone();

    // Prepare the data fetched from the CDK: Withdrawals + LBT
    // Withdrawals
    let withdraw_0_to_1 = vec![make_tx(0, 1, &eth, 10), make_tx(0, 1, &usdc, 100)];
    let withdraw_1_to_0 = vec![make_tx(1, 0, &eth, 20), make_tx(1, 0, &usdc, 200)];

    vec![
        Certificate::new(
            0.into(),
            Default::default(),
            Default::default(),
            Default::default(),
            withdraw_0_to_1.clone(),
        ),
        Certificate::new(
            1.into(),
            Default::default(),
            Default::default(),
            Default::default(),
            withdraw_1_to_0.clone(),
        ),
    ]
}

#[rstest]
fn should_detect_debtor(initial_state: State, mut state_transition: Vec<Certificate>) {
    // Compute the full proof
    assert!(matches!(
        generate_full_proof_with_state(initial_state, &mut state_transition),
        Err(ProofError::NotEnoughBalance { .. })
    ));
}
// Success case
// {
//     // Initial balances for the CDKs
//     let initial_0 = BalanceTree::from(vec![deposit_eth(12), deposit_usdc(102)]);
//     let initial_1 = BalanceTree::from(vec![deposit_eth(20), deposit_usdc(201)]);

//     let certificates = vec![
//         Certificate::new(
//             0.into(),
//             dummy.clone(),
//             dummy_root.clone(),
//             initial_0,
//             withdraw_0_to_1.clone(),
//         ),
//         Certificate::new(1.into(), dummy, dummy_root, initial_1, withdraw_1_to_0.clone()),
//     ];

//     // Compute the full proof
//     assert!(generate_full_proof(&certificates).is_ok());
// }

#[test]
#[ignore = "not implemented yet"]
fn test_full_proof_mainnet_data() {
    // from data fetched from mainnet
}
