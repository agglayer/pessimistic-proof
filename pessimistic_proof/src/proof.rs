use std::{
    collections::{BTreeMap, HashMap},
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::{
    batch::{BalanceTree, Batch},
    keccak::Digest,
    withdrawal::NetworkId,
    Withdrawal,
};

/// Records all the deposits and withdrawals for each network.
///
/// Specifically, this records a map `network => (token_id => (deposit, withdraw))`: for each
/// network, the amounts withdrawn and deposited for every token are recorded.
///
/// Note: a "deposit" is the counterpart of a [`Withdrawal`]; a "withdrawal" from the source
/// network is a "deposit" in the destination network.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceTreeByNetwork(BTreeMap<NetworkId, BalanceTree>);

impl BalanceTreeByNetwork {
    /// Creates a new empty [`BalanceTreeByNetwork`].
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Updates the origin and destination network in the aggregate from a [`Withdrawal`].
    pub fn insert(&mut self, origin_network: NetworkId, withdrawal: Withdrawal) {
        // Withdraw the origin network
        self.0
            .entry(origin_network)
            .or_default()
            .withdraw(withdrawal.token_info.clone(), withdrawal.amount);

        // Deposit the destination network
        self.0
            .entry(withdrawal.dest_network)
            .or_default()
            .deposit(withdrawal.token_info, withdrawal.amount);
    }

    /// Merge two [`BalanceTreeByNetwork`].
    pub fn merge(&mut self, other: &BalanceTreeByNetwork) {
        for (network, balance_tree) in other.0.iter() {
            self.0
                .entry(*network)
                .and_modify(|bt| bt.merge(balance_tree))
                .or_insert(balance_tree.clone());
        }
    }
}

impl From<BTreeMap<NetworkId, BalanceTree>> for BalanceTreeByNetwork {
    fn from(value: BTreeMap<NetworkId, BalanceTree>) -> Self {
        Self(value)
    }
}

impl Deref for BalanceTreeByNetwork {
    type Target = BTreeMap<NetworkId, BalanceTree>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BalanceTreeByNetwork {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Represents all errors that can occur while generating the proof.
#[derive(Debug)]
pub enum ProofError {
    InvalidLocalExitRoot { got: Digest, expected: Digest },
    NotEnoughBalance { debtors: Vec<NetworkId> },
}

/// Flatten the [`BalanceTreeByNetwork`] across all batches.
pub fn merge_balance_trees(
    aggregates: &HashMap<NetworkId, BalanceTreeByNetwork>,
) -> BalanceTreeByNetwork {
    let mut collated = BalanceTreeByNetwork::new();

    for aggregate in aggregates.values() {
        collated.merge(aggregate);
    }

    collated
}

pub type ExitRoot = Digest;
pub type BalanceRoot = Digest;

/// Returns the updated local balance and exit roots for each network.
pub fn generate_full_proof(
    batches: &[Batch],
) -> Result<(HashMap<NetworkId, ExitRoot>, HashMap<NetworkId, BalanceRoot>), ProofError> {
    // Check the validity of the provided exit roots
    for batch in batches {
        let computed_root = batch.prev_local_exit_tree.get_root();

        if computed_root != batch.prev_local_exit_root {
            return Err(ProofError::InvalidLocalExitRoot {
                got: computed_root,
                expected: batch.prev_local_exit_root,
            });
        }
    }

    // Compute the new exit root
    let exit_roots: HashMap<NetworkId, ExitRoot> = batches
        .iter()
        .map(|batch| (batch.origin_network, batch.compute_new_exit_root()))
        .collect();

    // Compute the new balance tree by network
    let balance_trees: HashMap<NetworkId, BalanceTreeByNetwork> = batches
        .iter()
        .map(|batch| (batch.origin_network, batch.compute_new_balance_tree()))
        .collect();

    // Merge the balance tree by network
    let collated: BalanceTreeByNetwork = merge_balance_trees(&balance_trees);

    // Detect the debtors if any
    let debtors = collated
        .iter()
        .filter(|(_, balance_tree)| balance_tree.has_debt())
        .map(|(network, _)| *network)
        .collect::<Vec<_>>();

    if !debtors.is_empty() {
        return Err(ProofError::NotEnoughBalance { debtors });
    }

    let balance_roots: HashMap<NetworkId, BalanceRoot> = collated
        .iter()
        .map(|(network, balance_tree)| (*network, balance_tree.hash()))
        .collect();

    Ok((exit_roots, balance_roots))
}
