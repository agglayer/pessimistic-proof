use poly_pessimistic_proof::{
    batch::{Amount, Balance, BalanceTree, Batch},
    generate_full_proof,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    FinalProofError, TokenInfo, Withdrawal,
};
use reth_primitives::{address, U256};

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

#[test]
fn test_full_proof() {
    let eth = TokenInfo {
        origin_network: 0.into(),
        origin_token_address: address!("0000000000000000000000000000000000000000"),
    };

    let usdc = TokenInfo {
        origin_network: 0.into(),
        origin_token_address: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
    };

    let dummy: LocalExitTree<Keccak256Hasher> =
        LocalExitTree::from_leaves([[0_u8; 32], [1_u8; 32], [2_u8; 32]].into_iter());
    let dummy_root = dummy.get_root();

    // Prepare the data fetched from the CDK: Withdrawals + LBT

    // Withdrawals
    let withdraw_0_to_1 = vec![make_tx(0, 1, &eth, 10), make_tx(0, 1, &usdc, 100)];
    let withdraw_1_to_0 = vec![make_tx(1, 0, &eth, 20), make_tx(1, 0, &usdc, 200)];

    let deposit = |v: u32| Balance::new(Amount::Deposit(U256::from(v)));

    // Failing case
    {
        // Initial balances for the CDKs
        let initial_0 =
            BalanceTree::new(vec![(eth.clone(), deposit(2)), (usdc.clone(), deposit(10))]);
        let initial_1 =
            BalanceTree::new(vec![(eth.clone(), deposit(1)), (usdc.clone(), deposit(200))]);

        let batches = vec![
            Batch::new(
                0.into(),
                dummy.clone(),
                dummy_root.clone(),
                initial_0,
                withdraw_0_to_1.clone(),
            ),
            Batch::new(
                1.into(),
                dummy.clone(),
                dummy_root.clone(),
                initial_1,
                withdraw_1_to_0.clone(),
            ),
        ];

        // Compute the full proof
        assert!(matches!(
            generate_full_proof(batches),
            Err(FinalProofError::NotEnoughBalance { .. })
        ));
    }

    // Success case
    {
        // Initial balances for the CDKs
        let initial_0 =
            BalanceTree::new(vec![(eth.clone(), deposit(12)), (usdc.clone(), deposit(102))]);
        let initial_1 =
            BalanceTree::new(vec![(eth.clone(), deposit(20)), (usdc.clone(), deposit(201))]);

        let batches = vec![
            Batch::new(
                0.into(),
                dummy.clone(),
                dummy_root.clone(),
                initial_0,
                withdraw_0_to_1.clone(),
            ),
            Batch::new(1.into(), dummy, dummy_root, initial_1, withdraw_1_to_0.clone()),
        ];

        // Compute the full proof
        assert!(generate_full_proof(batches).is_ok());
    }
}

#[test]
#[ignore = "not implemented yet"]
fn test_full_proof_mainnet_data() {
    // from data fetched from mainnet
}
