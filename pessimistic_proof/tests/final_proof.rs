use poly_invariant_proof::{
    batch::{Batch, LocalCreditTree},
    generate_jumbo_proof, TokenInfo, Withdrawal,
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
fn test_final_proof() {
    let eth = TokenInfo {
        origin_network: 0.into(),
        origin_token_address: address!("0000000000000000000000000000000000000000"),
    };

    let usdc = TokenInfo {
        origin_network: 0.into(),
        origin_token_address: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
    };

    // Prepare the data fetched from the CDK: Withdrawals + LBT

    // Withdrawals
    let withdraw_0_to_1 = vec![make_tx(0, 1, &eth, 100), make_tx(0, 1, &usdc, 1000)];
    let withdraw_1_to_0 = vec![make_tx(1, 0, &eth, 200), make_tx(1, 0, &usdc, 2000)];

    // Success case
    {
        // Initial balances for the CDKs
        let initial_0 = LocalCreditTree::new(vec![(eth.clone(), U256::from(300))]);
        let initial_1 = LocalCreditTree::new(vec![(eth.clone(), U256::from(200))]);

        let batches = vec![
            Batch::new(0.into(), initial_0, withdraw_0_to_1.clone()),
            Batch::new(1.into(), initial_1, withdraw_1_to_0.clone()),
        ];

        // Compute the jumbo proof
        assert!(generate_jumbo_proof(batches).is_ok());
    }

    // Failing case
    {
        // Initial balances for the CDKs
        let initial_0 = LocalCreditTree::new(vec![(eth.clone(), U256::from(20))]);
        let initial_1 = LocalCreditTree::new(vec![(eth.clone(), U256::from(5))]);

        let _batches = vec![
            Batch::new(0.into(), initial_0, withdraw_0_to_1.clone()),
            Batch::new(1.into(), initial_1, withdraw_1_to_0.clone()),
        ];

        // TODO: Need check balance
        // Compute the jumbo proof
        // assert!(generate_jumbo_proof(batches).is_err());
    }
}

#[test]
#[ignore = "not implemented yet"]
fn test_final_proof_mainnet_data() {
    // from data fetched from mainnet
}
