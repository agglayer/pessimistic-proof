use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    certificate::Certificate,
    local_balance_tree::BalanceTreeByNetwork,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    proof::{BalanceRoot, ExitRoot},
    FullProofOutput, NetworkId, ProofError,
};

/// Represents the global state tracked by the AggLayer.
#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    pub global_exit_tree: BTreeMap<NetworkId, LocalExitTree<Keccak256Hasher>>,
    pub global_balance_tree: BalanceTreeByNetwork,
}

impl State {
    pub fn get_checkpoint(&self) -> FullProofOutput {
        let ger: HashMap<NetworkId, ExitRoot> = self
            .global_exit_tree
            .iter()
            .map(|(network, exit_tree)| (*network, exit_tree.get_root()))
            .collect();

        let gbr: HashMap<NetworkId, BalanceRoot> = self
            .global_balance_tree
            .iter()
            .map(|(network, balance_tree)| (*network, balance_tree.hash()))
            .collect();

        (ger, gbr)
    }

    /// Apply the [`Certificate`] on the current [`State`].
    /// Returns the new [`ExitRoot`] if successful write.
    pub fn apply_certificate(&mut self, certificate: Certificate) -> Result<ExitRoot, ProofError> {
        let origin_network = certificate.origin_network;

        // Apply on Exit Tree
        let new_local_exit_tree = {
            let mut local_exit_tree =
                self.global_exit_tree.get(&origin_network).expect("unknown").clone();
            let computed_root = local_exit_tree.get_root();
            if computed_root != certificate.prev_local_exit_root {
                return Err(ProofError::InvalidLocalExitRoot {
                    got: computed_root,
                    expected: certificate.prev_local_exit_root,
                });
            }

            for withdrawal in &certificate.withdrawals {
                local_exit_tree.add_leaf(withdrawal.hash());
            }

            local_exit_tree
        };

        // Apply on Balance Tree
        let new_balance_tree_by_network = {
            let mut new_balance_tree_by_network = self.global_balance_tree.clone();

            for withdrawal in &certificate.withdrawals {
                new_balance_tree_by_network.insert(certificate.origin_network, withdrawal.clone());
            }

            new_balance_tree_by_network
        };

        // Check whether the sender has some debt
        if let Some(balance_tree) = new_balance_tree_by_network.get(&certificate.origin_network) {
            if balance_tree.has_debt() {
                return Err(ProofError::HasDebt {
                    network: certificate.origin_network,
                });
            }
        };

        // All good, let's apply on the globals
        self.global_exit_tree
            .entry(origin_network)
            .and_modify(|current_let| *current_let = new_local_exit_tree.clone());

        self.global_balance_tree = new_balance_tree_by_network;

        Ok(new_local_exit_tree.get_root())
    }

    pub fn apply_certificates_from(
        &mut self,
        _origin_network: NetworkId,
        certificates: Vec<Certificate>,
    ) -> Result<(), ProofError> {
        // TODO: Check linkage among them
        for c in certificates {
            self.apply_certificate(c)?;
        }

        Ok(())
    }
}
