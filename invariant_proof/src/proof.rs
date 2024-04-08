use std::{collections::BTreeMap, ops::Deref};

use num_bigint::BigInt;
use reth_primitives::Address;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::local_exit_tree::{
    hasher::keccak::{Keccak256Hasher, KeccakDigest},
    withdrawal::Withdrawal,
    LocalExitTree,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateDeposits(BTreeMap<u32, BTreeMap<Address, BigInt>>);

impl AggregateDeposits {
    pub fn new(aggregate_deposits: BTreeMap<u32, BTreeMap<Address, BigInt>>) -> Self {
        Self(aggregate_deposits)
    }

    pub fn hash(&self) -> KeccakDigest {
        let mut hasher = Keccak::v256();

        for (dest_network, token_map) in self.0.iter() {
            hasher.update(&dest_network.to_be_bytes());

            for (token_id, amount) in token_map {
                hasher.update(token_id.as_slice());
                hasher.update(&amount.to_signed_bytes_be());
            }
        }

        let mut output = [0u8; 32];
        hasher.finalize(&mut output);
        output
    }
}

impl Deref for AggregateDeposits {
    type Target = BTreeMap<u32, BTreeMap<Address, BigInt>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
    prev_local_exit_tree: LocalExitTree<KeccakDigest, Keccak256Hasher>,
    prev_local_exit_root: KeccakDigest,
    withdrawals: Vec<Withdrawal>,
) -> Result<(KeccakDigest, AggregateDeposits), LeafProofError> {
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
    let mut aggregate_deposits = BTreeMap::new();

    for withdrawal in withdrawals {
        new_local_exit_tree.add_leaf(withdrawal.hash());

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

    Ok((new_local_exit_tree.get_root(), AggregateDeposits::new(aggregate_deposits)))
}
