use crate::{
    hasher::Keccak256Hasher,
    local_exit_tree::{withdrawal::Withdrawal, LocalExitTree},
};

#[derive(Debug)]
pub enum LeafProofError {
    InvalidLocalExitRoot { got: [u8; 32], expected: [u8; 32] },
}

/// Returns the root of the local exit tree resulting from adding every withdrawal to the previous
/// local exit tree
pub fn leaf_proof(
    prev_local_exit_tree: LocalExitTree<Keccak256Hasher>,
    prev_local_exit_root: [u8; 32],
    withdrawals: Vec<Withdrawal>,
) -> Result<[u8; 32], LeafProofError> {
    {
        let computed_root = prev_local_exit_tree.get_root();

        if computed_root != prev_local_exit_root {
            return Err(LeafProofError::InvalidLocalExitRoot {
                got: computed_root,
                expected: prev_local_exit_root,
            });
        }
    }

    let mut new_tree = prev_local_exit_tree;
    for withdrawal in withdrawals {
        new_tree.add_leaf(withdrawal.hash());
    }

    Ok(new_tree.get_root())
}
