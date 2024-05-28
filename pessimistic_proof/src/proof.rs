use std::collections::HashMap;

use crate::{
    batch::Batch,
    keccak::Digest,
    local_balance_tree::{merge_balance_trees, BalanceTreeByNetwork},
    withdrawal::NetworkId,
};

/// Represents all errors that can occur while generating the proof.
#[derive(Debug)]
pub enum ProofError {
    InvalidLocalExitRoot { got: Digest, expected: Digest },
    NotEnoughBalance { debtors: Vec<NetworkId> },
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
