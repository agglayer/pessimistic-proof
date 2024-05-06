use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
};

use reth_primitives::U256;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::{
    batch::{Batch, LocalCreditTree},
    keccak::Digest,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    withdrawal::{NetworkId, TokenInfo},
    Withdrawal,
};

/// Records all the deposits made in destination networks.
///
/// Specifically, this records a map `destination_network => (token_id => amount)`: for each
/// network, the amount deposited for every token is recorded.
///
/// Note: a "deposit" is the counterpart of a [`Withdrawal`]; a "withdrawal" from the source
/// network is a "deposit" in the destination network.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregateDeposits(BTreeMap<NetworkId, BTreeMap<TokenInfo, U256>>);

impl AggregateDeposits {
    /// Creates a new empty [`AggregateDeposits`].
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Updates the aggregate deposits from a [`Withdrawal`] (representing a withdrawal from the
    /// source network).
    pub fn insert(&mut self, withdrawal: Withdrawal) {
        let token_info = withdrawal.token_info;

        self.0
            .entry(withdrawal.dest_network)
            .and_modify(|network_map: &mut BTreeMap<TokenInfo, U256>| {
                network_map
                    .entry(token_info.clone())
                    .and_modify(|current_amount| *current_amount += withdrawal.amount)
                    .or_insert_with(|| withdrawal.amount);
            })
            .or_insert_with(|| {
                BTreeMap::from_iter(std::iter::once((token_info, withdrawal.amount)))
            });
    }

    /// Returns the hash of [`AggregateDeposits`].
    pub fn hash(&self) -> Digest {
        let mut hasher = Keccak::v256();

        for (dest_network, token_map) in self.0.iter() {
            hasher.update(&dest_network.to_be_bytes());

            for (token_info, amount) in token_map {
                hasher.update(&token_info.hash());
                hasher.update(&amount.to_be_bytes::<32>());
            }
        }

        let mut output = [0u8; 32];
        hasher.finalize(&mut output);
        output
    }

    /// Returns the [`LocalCreditTree`] which sums up the deposits on each token across
    /// all networks.
    pub fn flatten(&self) -> LocalCreditTree {
        let mut flatten = LocalCreditTree::default();
        for (_, token_map) in self.0.iter() {
            for (token_info, amount) in token_map {
                flatten.add_credit(token_info, amount);
            }
        }
        flatten
    }
}

impl Deref for AggregateDeposits {
    type Target = BTreeMap<NetworkId, BTreeMap<TokenInfo, U256>>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
    prev_local_exit_tree: LocalExitTree<Keccak256Hasher>,
    prev_local_exit_root: Digest,
    withdrawals: Vec<Withdrawal>,
) -> Result<(Digest, AggregateDeposits), LeafProofError> {
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
    let mut aggregate_deposits = AggregateDeposits::new();

    for withdrawal in withdrawals {
        new_local_exit_tree.add_leaf(withdrawal.hash());
        aggregate_deposits.insert(withdrawal);
    }

    Ok((new_local_exit_tree.get_root(), aggregate_deposits))
}

/// Represents all errors that can occur while generating the final proof.
#[derive(Debug)]
pub enum FinalProofError {
    UnknownToken,
    NotEnoughBalance {
        network: NetworkId,
        token: TokenInfo,
    },
}

/// Given an origin, how much was sent, to each target network, for each token.
pub fn create_aggregated_deposits(batches: &Vec<Batch>) -> HashMap<NetworkId, AggregateDeposits> {
    // TODO: Take the exit trees from the batch
    let dummy: LocalExitTree<Keccak256Hasher> =
        LocalExitTree::from_leaves([[0_u8; 32], [1_u8; 32], [2_u8; 32]].into_iter());
    let dummy_root = dummy.get_root();

    let mut aggregated_deposits: HashMap<NetworkId, AggregateDeposits> =
        HashMap::with_capacity(batches.len());

    for batch in batches {
        // TODO: Handle failures
        let (_digest, deposits) =
            generate_leaf_proof(dummy.clone(), dummy_root, batch.withdrawals.clone())
                .ok()
                .unwrap();
        aggregated_deposits.insert(batch.origin, deposits);
    }

    aggregated_deposits
}

/// Given a target, how much was received, for each token.
pub fn create_collated_deposits(
    aggregated_deposits: &HashMap<NetworkId, AggregateDeposits>,
) -> HashMap<NetworkId, LocalCreditTree> {
    let mut collated_deposits = HashMap::with_capacity(aggregated_deposits.len());

    for deposits in aggregated_deposits.values() {
        for (target, token_map) in deposits.iter() {
            for (token_info, amount) in token_map {
                collated_deposits
                    .entry(*target)
                    .or_insert_with(LocalCreditTree::default)
                    .add_credit(token_info, amount);
            }
        }
    }

    collated_deposits
}

/// Returns the updated local balance tree for each network.
pub fn generate_jumbo_proof(
    batches: Vec<Batch>,
) -> Result<HashMap<NetworkId, LocalCreditTree>, FinalProofError> {
    let aggregated_deposits: HashMap<NetworkId, AggregateDeposits> =
        create_aggregated_deposits(&batches);

    let mut collated_deposits: HashMap<NetworkId, LocalCreditTree> =
        create_collated_deposits(&aggregated_deposits);

    let _outbound_transfers: HashMap<NetworkId, LocalCreditTree> = aggregated_deposits
        .iter()
        .map(|(network, deposits)| (*network, deposits.flatten()))
        .collect::<HashMap<_, _>>();

    // Update all the local credit tree
    for batch in batches {
        collated_deposits
            .entry(batch.origin)
            .and_modify(|deposits| deposits.merge(batch.local_credit_tree));
    }

    // TODO: Check balances

    Ok(collated_deposits)
}
