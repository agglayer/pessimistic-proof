use std::{
    collections::{BTreeMap, HashMap},
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::{
    batch::{BalanceTree, Batch},
    keccak::Digest,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    withdrawal::NetworkId,
    Withdrawal,
};

/// Records all the deposits and withdrawals for each network.
///
/// Specifically, this records a map `destination_network => (token_id => (credit, debit))`: for each
/// network, the amounts withdrawn and deposited for every token are recorded.
///
/// Note: a "deposit" is the counterpart of a [`Withdrawal`]; a "withdrawal" from the source
/// network is a "deposit" in the destination network.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Aggregate(BTreeMap<NetworkId, BalanceTree>);

impl Aggregate {
    /// Creates a new empty [`Aggregate`].
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Creates a new non-empty [`Aggregate`].
    pub fn new_with(base: BTreeMap<NetworkId, BalanceTree>) -> Self {
        Self(base)
    }

    /// Updates the aggregate deposits from a [`Withdrawal`] (representing a withdrawal from the
    /// source network).
    pub fn insert(&mut self, origin_network: NetworkId, withdrawal: Withdrawal) {
        // Debit the origin network
        self.0
            .entry(origin_network)
            .or_default()
            .debit(&withdrawal.token_info, &withdrawal.amount);

        // Credit the destination network
        self.0
            .entry(withdrawal.dest_network)
            .or_default()
            .credit(&withdrawal.token_info, &withdrawal.amount);
    }

    /// Returns the hash of [`Aggregate`].
    pub fn hash(&self) -> Digest {
        let mut hasher = Keccak::v256();

        for (dest_network, balance_tree) in self.0.iter() {
            hasher.update(&dest_network.to_be_bytes());

            for (token_info, balance) in balance_tree.balances.iter() {
                hasher.update(&token_info.hash());
                hasher.update(&balance.hash());
            }
        }

        let mut output = [0u8; 32];
        hasher.finalize(&mut output);
        output
    }

    /// Merge two [`Aggregate`].
    pub fn merge(&mut self, other: &Aggregate) {
        for (network, balance_tree) in other.0.iter() {
            self.0
                .entry(*network)
                .and_modify(|bt| bt.merge(balance_tree.clone()))
                .or_insert(balance_tree.clone());
        }
    }
}

impl Deref for Aggregate {
    type Target = BTreeMap<NetworkId, BalanceTree>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Aggregate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Represents all errors that can occur while generating the leaf proof.
#[derive(Debug)]
pub enum LeafProofError {
    InvalidLocalExitRoot { got: Digest, expected: Digest },
}

/// Returns the root of the local exit tree resulting from adding every withdrawal to the previous
/// local exit tree, as well as a record of all deposits made.
pub fn generate_leaf_proof(
    origin_network: NetworkId,
    local_balance_tree: BalanceTree,
    prev_local_exit_tree: LocalExitTree<Keccak256Hasher>,
    prev_local_exit_root: Digest,
    withdrawals: Vec<Withdrawal>,
) -> Result<(Digest, Aggregate), LeafProofError> {
    {
        let computed_root = prev_local_exit_tree.get_root();

        if computed_root != prev_local_exit_root {
            return Err(LeafProofError::InvalidLocalExitRoot {
                got: computed_root,
                expected: prev_local_exit_root,
            });
        }
    }

    let mut new_local_exit_tree = prev_local_exit_tree;

    let mut base = BTreeMap::new();
    base.insert(origin_network, local_balance_tree);
    let mut aggregate = Aggregate::new_with(base);

    for withdrawal in withdrawals {
        new_local_exit_tree.add_leaf(withdrawal.hash());
        aggregate.insert(origin_network, withdrawal.clone());
    }

    Ok((new_local_exit_tree.get_root(), aggregate))
}

/// Represents all errors that can occur while generating the final proof.
#[derive(Debug)]
pub enum FinalProofError {
    UnknownToken,
    NotEnoughBalance { debtor: Vec<NetworkId> },
}

// Generate the [`Aggregate`] for each Batch.
pub fn create_aggregates(batches: &Vec<Batch>) -> HashMap<NetworkId, Aggregate> {
    // TODO: Take the exit trees from the batch
    let dummy: LocalExitTree<Keccak256Hasher> =
        LocalExitTree::from_leaves([[0_u8; 32], [1_u8; 32], [2_u8; 32]].into_iter());
    let dummy_root = dummy.get_root();

    let mut aggregated_deposits: HashMap<NetworkId, Aggregate> =
        HashMap::with_capacity(batches.len());

    for batch in batches {
        // TODO: Handle failures
        let (_digest, aggregate) = generate_leaf_proof(
            batch.origin,
            batch.local_balance_tree.clone(),
            dummy.clone(),
            dummy_root,
            batch.withdrawals.clone(),
        )
        .ok()
        .unwrap();
        aggregated_deposits.insert(batch.origin, aggregate);
    }

    aggregated_deposits
}

/// Flatten the [`Aggregate`] across all batches.
pub fn create_collation(aggregates: &HashMap<NetworkId, Aggregate>) -> Aggregate {
    let mut collated = Aggregate::new();

    for aggregate in aggregates.values() {
        collated.merge(aggregate);
    }

    collated
}

/// Returns the updated local balance tree for each network.
pub fn generate_jumbo_proof(batches: Vec<Batch>) -> Result<Aggregate, FinalProofError> {
    let aggregates: HashMap<NetworkId, Aggregate> = create_aggregates(&batches);
    let mut collated: Aggregate = create_collation(&aggregates);

    // Detect the cheaters if any
    let debtor = collated
        .iter()
        .filter(|(_, aggregate)| aggregate.has_debt())
        .map(|(network, _)| network)
        .cloned()
        .collect::<Vec<_>>();

    if !debtor.is_empty() {
        return Err(FinalProofError::NotEnoughBalance { debtor });
    }

    // Update the balances
    for balance_tree in collated.values_mut() {
        balance_tree.apply_debit();
    }

    Ok(collated)
}
