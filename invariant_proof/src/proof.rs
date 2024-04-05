use std::collections::BTreeMap;

use num_bigint::BigInt;
use reth_primitives::Address;

use crate::{
    hasher::keccak::{Keccak256Hasher, KeccakDigest},
    local_exit_tree::{withdrawal::Withdrawal, LocalExitTree},
};

pub type AggregateDeposits = BTreeMap<u32, BTreeMap<Address, BigInt>>;

#[derive(Debug)]
pub enum LeafProofError {
    InvalidLocalExitRoot {
        got: KeccakDigest,
        expected: KeccakDigest,
    },
}

/// Returns the root of the local exit tree resulting from adding every withdrawal to the previous
/// local exit tree
pub fn leaf_proof(
    prev_local_exit_tree: LocalExitTree<KeccakDigest>,
    prev_local_exit_root: KeccakDigest,
    withdrawals: Vec<Withdrawal>,
) -> Result<(KeccakDigest, AggregateDeposits), LeafProofError> {
    {
        let computed_root = prev_local_exit_tree.get_root::<Keccak256Hasher>();

        if computed_root != prev_local_exit_root {
            return Err(LeafProofError::InvalidLocalExitRoot {
                got: computed_root,
                expected: prev_local_exit_root,
            });
        }
    }

    let mut new_local_exit_tree = prev_local_exit_tree;
    let mut aggregate_deposits = BTreeMap::new();

    for withdrawal in withdrawals {
        new_local_exit_tree.add_leaf::<Keccak256Hasher>(withdrawal.hash());

        // FIXME: This incorrectly uses `Withdrawal.dest_address` as the token identifier
        let withdrawal_amount = withdrawal.amount.clone();
        aggregate_deposits
            .entry(withdrawal.dest_network)
            .and_modify(|network_map: &mut BTreeMap<Address, BigInt>| {
                network_map
                    .entry(withdrawal.dest_address)
                    .and_modify(|current_amount| *current_amount += withdrawal.amount)
                    .or_insert_with(|| withdrawal_amount.clone());
            })
            .or_insert(BTreeMap::from_iter(std::iter::once((
                withdrawal.dest_address,
                withdrawal_amount,
            ))));
    }

    Ok((new_local_exit_tree.get_root::<Keccak256Hasher>(), aggregate_deposits))
}
