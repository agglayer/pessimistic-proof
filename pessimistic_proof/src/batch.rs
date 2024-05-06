use std::collections::BTreeMap;

use reth_primitives::U256;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    withdrawal::{NetworkId, TokenInfo},
    Withdrawal,
};

/// Records the balances for each [`TokenInfo`] for one network.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LocalCreditTree {
    /// Balances for each token
    pub(crate) balances: BTreeMap<TokenInfo, U256>,
}

impl LocalCreditTree {
    pub fn new(initial_balance: Vec<(TokenInfo, U256)>) -> Self {
        Self {
            balances: initial_balance.into_iter().collect(),
        }
    }

    /// Add Credit to [`TokenInfo`].
    pub fn add_credit(&mut self, token: &TokenInfo, credit: &U256) {
        self.balances
            .entry(token.clone())
            .and_modify(|balance| *balance += *credit)
            .or_insert(*credit);
    }

    /// Merge with another [`LocalCreditTree`].
    pub fn merge(&mut self, other: LocalCreditTree) {
        for (token, amount) in other.balances.iter() {
            self.add_credit(token, amount);
        }
    }
}

/// Represents a batch submitted by CDKs to the AggLayer.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Batch {
    /// Origin network
    pub(crate) origin: NetworkId,
    /// Initial state considered for the batch
    pub(crate) local_credit_tree: LocalCreditTree,
    /// Set of withdrawals for the batch
    pub(crate) withdrawals: Vec<Withdrawal>,
}

impl Batch {
    /// Creates a new [`Batch`].
    pub fn new(
        origin: NetworkId,
        initial_balance: LocalCreditTree,
        withdrawals: Vec<Withdrawal>,
    ) -> Self {
        Self {
            origin,
            local_credit_tree: initial_balance,
            withdrawals,
        }
    }
}
