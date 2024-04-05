use crate::{
    hasher::keccak::{Keccak256Hasher, KeccakDigest},
    local_exit_tree::{withdrawal::Withdrawal, LocalExitTree},
};

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
) -> Result<KeccakDigest, LeafProofError> {
    {
        let computed_root = prev_local_exit_tree.get_root::<Keccak256Hasher>();

        if computed_root != prev_local_exit_root {
            return Err(LeafProofError::InvalidLocalExitRoot {
                got: computed_root,
                expected: prev_local_exit_root,
            });
        }
    }

    let mut new_tree = prev_local_exit_tree;
    for withdrawal in withdrawals {
        new_tree.add_leaf::<Keccak256Hasher>(withdrawal.hash());
    }

    Ok(new_tree.get_root::<Keccak256Hasher>())
}
