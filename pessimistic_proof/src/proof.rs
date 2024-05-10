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
pub struct Aggregate(BTreeMap<NetworkId, BalanceTree>);

pub type ExitRoots = HashMap<NetworkId, Digest>;
pub type BalanceRoots = HashMap<NetworkId, Digest>;

impl Aggregate {
    /// Creates a new empty [`Aggregate`].
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Updates the origin and destination network in the aggregate from a [`Withdrawal`].
    pub fn insert(&mut self, origin_network: NetworkId, withdrawal: Withdrawal) {
        // Withdraw the origin network
        self.0
            .entry(origin_network)
            .or_default()
            .withdraw(&withdrawal.token_info, &withdrawal.amount);

        // Deposit the destination network
        self.0
            .entry(withdrawal.dest_network)
            .or_default()
            .deposit(&withdrawal.token_info, &withdrawal.amount);
    }

    /// Merge two [`Aggregate`].
    pub fn merge(&mut self, other: &Aggregate) {
        for (network, balance_tree) in other.0.iter() {
            self.0
                .entry(*network)
                .and_modify(|bt| bt.merge(&balance_tree.balances))
                .or_insert(balance_tree.clone());
        }
    }
}

impl From<BTreeMap<NetworkId, BalanceTree>> for Aggregate {
    fn from(value: BTreeMap<NetworkId, BalanceTree>) -> Self {
        Self(value)
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

/// Represents all errors that can occur while generating the proof.
#[derive(Debug)]
pub enum ProofError {
    InvalidLocalExitRoot { got: Digest, expected: Digest },
    NotEnoughBalance { debtors: Vec<NetworkId> },
}

/// Returns the root of the local exit tree resulting from adding every withdrawal to the previous
/// local exit tree, as well as a record of all witdrawals and deposits made.
pub fn generate_leaf_proof(batch: Batch) -> Result<(Digest, Aggregate), ProofError> {
    {
        let computed_root = batch.prev_local_exit_tree.get_root();

        if computed_root != batch.prev_local_exit_root {
            return Err(ProofError::InvalidLocalExitRoot {
                got: computed_root,
                expected: batch.prev_local_exit_root,
            });
        }
    }

    let mut new_local_exit_tree = batch.prev_local_exit_tree;

    let mut aggregate: Aggregate = {
        let base: BTreeMap<NetworkId, BalanceTree> =
            [(batch.origin_network, batch.prev_local_balance_tree)].into();
        base.into()
    };

    for withdrawal in batch.withdrawals {
        new_local_exit_tree.add_leaf(withdrawal.hash());
        aggregate.insert(batch.origin_network, withdrawal.clone());
    }

    Ok((new_local_exit_tree.get_root(), aggregate))
}

// Generate the [`Aggregate`] for each Batch.
pub fn create_aggregates(
    batches: &[Batch],
) -> Result<(ExitRoots, HashMap<NetworkId, Aggregate>), ProofError> {
    let mut aggregates = HashMap::with_capacity(batches.len());
    let mut exit_roots = HashMap::with_capacity(batches.len());

    for batch in batches {
        let (digest, aggregate) = generate_leaf_proof(batch.clone())?;
        aggregates.insert(batch.origin_network, aggregate);
        exit_roots.insert(batch.origin_network, digest);
    }

    Ok((exit_roots, aggregates))
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
pub fn generate_full_proof(batches: &[Batch]) -> Result<(ExitRoots, BalanceRoots), ProofError> {
    let (exit_roots, aggregates) = create_aggregates(batches)?;
    let collated: Aggregate = create_collation(&aggregates);

    // Detect the cheaters if any
    let debtors = collated
        .iter()
        .filter(|(_, balance_tree)| balance_tree.has_debt())
        .map(|(network, _)| *network)
        .collect::<Vec<_>>();

    if !debtors.is_empty() {
        return Err(ProofError::NotEnoughBalance { debtors });
    }

    let balance_tree_roots = collated
        .iter()
        .map(|(network, balance_tree)| (*network, balance_tree.hash()))
        .collect();

    Ok((exit_roots, balance_tree_roots))
}
