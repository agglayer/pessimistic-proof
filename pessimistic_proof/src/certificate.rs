use serde::{Deserialize, Serialize};

use crate::{keccak::Digest, withdrawal::NetworkId, Withdrawal};

/// Represents the data submitted by the CDKs to the AggLayer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certificate {
    /// Origin network which emitted this certificate
    pub origin_network: NetworkId,
    /// Initial local exit root
    pub prev_local_exit_root: Digest,
    /// Set of withdrawals
    pub withdrawals: Vec<Withdrawal>,
}

impl Certificate {
    /// Creates a new [`Certificate`].
    pub fn new(
        origin_network: NetworkId,
        prev_local_exit_root: Digest,
        withdrawals: Vec<Withdrawal>,
    ) -> Self {
        Self {
            origin_network,
            prev_local_exit_root,
            withdrawals,
        }
    }
}

