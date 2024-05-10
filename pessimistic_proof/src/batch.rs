use std::collections::BTreeMap;

use reth_primitives::U256;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    keccak::Digest,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    withdrawal::{NetworkId, TokenInfo},
    Withdrawal,
};

/// Record the balance as total deposit and total withdraw.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Balance {
    deposit: U256,
    withdraw: U256,
}

pub struct Deposit(pub U256);
pub struct Withdraw(pub U256);

impl From<Deposit> for Balance {
    fn from(v: Deposit) -> Self {
        Self {
            deposit: v.0,
            withdraw: U256::ZERO,
        }
    }
}

impl From<Withdraw> for Balance {
    fn from(v: Withdraw) -> Self {
        Self {
            deposit: U256::ZERO,
            withdraw: v.0,
        }
    }
}

impl Balance {
    /// Returns the balance.
    pub fn balance(&self) -> U256 {
        self.deposit - self.withdraw
    }

    /// Returns whether the balance is negative.
    pub fn is_negative(&self) -> bool {
        self.withdraw > self.deposit
    }

    pub fn deposit(&mut self, amount: U256) {
        self.deposit += amount;
    }

    pub fn withdraw(&mut self, amount: U256) {
        self.withdraw += amount;
    }
}

/// Records the balances for each [`TokenInfo`].
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BalanceTree {
    /// Balances for each token
    pub(crate) balances: BTreeMap<TokenInfo, Balance>,
}

impl From<Vec<(TokenInfo, Balance)>> for BalanceTree {
    fn from(initial_balance: Vec<(TokenInfo, Balance)>) -> Self {
        Self {
            balances: initial_balance.into_iter().collect(),
        }
    }
}

impl BalanceTree {
    /// Apply deposit to [`TokenInfo`].
    pub fn deposit(&mut self, token: &TokenInfo, amount: &U256) {
        self.balances.entry(token.clone()).or_default().deposit(*amount);
    }

    /// Apply withdraw to [`TokenInfo`].
    pub fn withdraw(&mut self, token: &TokenInfo, amount: &U256) {
        self.balances.entry(token.clone()).or_default().withdraw(*amount);
    }

    /// Merge with another [`BalanceTree`].
    pub fn merge(&mut self, other: &BTreeMap<TokenInfo, Balance>) {
        for (token, balance) in other.iter() {
            self.deposit(token, &balance.deposit);
            self.withdraw(token, &balance.withdraw)
        }
    }

    /// Returns whether any token has debt.
    /// TODO: We may want to return the debtor (token, debt)
    pub fn has_debt(&self) -> bool {
        self.balances.iter().any(|(_, balance)| balance.is_negative())
    }
}

/// Represents a batch submitted by CDKs to the AggLayer.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Batch {
    /// Origin network which emitted this batch
    pub origin_network: NetworkId,
    /// Initial local exit tree
    pub prev_local_exit_tree: LocalExitTree<Keccak256Hasher>,
    /// Initial local exit root
    pub prev_local_exit_root: Digest,
    /// Initial balance tree
    pub prev_local_balance_tree: BalanceTree,
    /// Set of withdrawals
    pub withdrawals: Vec<Withdrawal>,
}

impl Batch {
    /// Creates a new [`Batch`].
    pub fn new(
        origin_network: NetworkId,
        prev_local_exit_tree: LocalExitTree<Keccak256Hasher>,
        prev_local_exit_root: Digest,
        prev_local_balance_tree: BalanceTree,
        withdrawals: Vec<Withdrawal>,
    ) -> Self {
        Self {
            origin_network,
            prev_local_exit_tree,
            prev_local_exit_root,
            prev_local_balance_tree,
            withdrawals,
        }
    }
}
