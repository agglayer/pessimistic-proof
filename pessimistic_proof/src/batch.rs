use std::collections::BTreeMap;

use reth_primitives::U256;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tiny_keccak::{Hasher, Keccak};

use crate::{
    keccak::Digest,
    withdrawal::{NetworkId, TokenInfo},
    Withdrawal,
};

/// Record the balance as total deposit and total withdraw.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Balance {
    deposit: U256,
    withdraw: U256,
}

pub enum Amount {
    Deposit(U256),
    Withdraw(U256),
}

impl Balance {
    pub fn new(amount: Amount) -> Self {
        match amount {
            Amount::Deposit(val) => Self {
                deposit: val,
                withdraw: U256::ZERO,
            },
            Amount::Withdraw(val) => Self {
                deposit: U256::ZERO,
                withdraw: val,
            },
        }
    }

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

    pub fn apply_withdraw(&mut self) {
        debug_assert!(!self.is_negative(), "negative balance");

        self.deposit -= self.withdraw;
        self.withdraw = U256::ZERO;
    }

    pub fn hash(&self) -> Digest {
        let mut hasher = Keccak::v256();

        hasher.update(&self.deposit.to_be_bytes::<32>());
        hasher.update(&self.withdraw.to_be_bytes::<32>());

        let mut output = [0u8; 32];
        hasher.finalize(&mut output);
        output
    }
}

/// Records the balances for each [`TokenInfo`].
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BalanceTree {
    /// Balances for each token
    pub(crate) balances: BTreeMap<TokenInfo, Balance>,
}

impl BalanceTree {
    pub fn new(initial_balance: Vec<(TokenInfo, Balance)>) -> Self {
        Self {
            balances: initial_balance.into_iter().collect(),
        }
    }

    /// Apply deposit to [`TokenInfo`].
    pub fn deposit(&mut self, token: &TokenInfo, amount: &U256) {
        self.balances.entry(token.clone()).or_default().deposit(*amount);
    }

    /// Apply withdraw to [`TokenInfo`].
    pub fn withdraw(&mut self, token: &TokenInfo, amount: &U256) {
        self.balances.entry(token.clone()).or_default().withdraw(*amount);
    }

    /// Merge with another [`BalanceTree`].
    pub fn merge(&mut self, other: BalanceTree) {
        for (token, balance) in other.balances.iter() {
            self.deposit(token, &balance.deposit);
            self.withdraw(token, &balance.withdraw)
        }
    }

    /// Returns whether any token has debt.
    /// TODO: We may want to return the debtor (token, debt)
    pub fn has_debt(&self) -> bool {
        self.balances.iter().any(|(_, balance)| balance.is_negative())
    }

    pub fn apply_withdraw(&mut self) {
        for balance in self.balances.values_mut() {
            balance.apply_withdraw();
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
    pub(crate) local_balance_tree: BalanceTree,
    /// Set of withdrawals for the batch
    pub(crate) withdrawals: Vec<Withdrawal>,
}

impl Batch {
    /// Creates a new [`Batch`].
    pub fn new(
        origin: NetworkId,
        initial_balance: BalanceTree,
        withdrawals: Vec<Withdrawal>,
    ) -> Self {
        Self {
            origin,
            local_balance_tree: initial_balance,
            withdrawals,
        }
    }
}
