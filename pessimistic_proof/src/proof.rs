use std::collections::{BTreeMap, HashMap};

use crate::{certificate::Certificate, keccak::Digest, withdrawal::NetworkId, State};

/// Represents all errors that can occur while generating the proof.
#[derive(Debug)]
pub enum ProofError {
    InvalidLocalExitRoot { got: Digest, expected: Digest },
    NotEnoughBalance { debtors: Vec<NetworkId> },
    HasDebt { network: NetworkId },
}

pub type ExitRoot = Digest;
pub type BalanceRoot = Digest;
pub type FullProofOutput = (HashMap<NetworkId, ExitRoot>, HashMap<NetworkId, BalanceRoot>);

pub fn generate_full_proof_with_state(
    initial_state: State,
    certificates: Vec<Certificate>,
) -> Result<FullProofOutput, ProofError> {
    // Apply all certificates per network bucket
    let mut certificate_by_network: BTreeMap<NetworkId, Vec<Certificate>> = BTreeMap::new();
    for certificate in certificates {
        certificate_by_network
            .entry(certificate.origin_network)
            .or_default()
            .push(certificate);
    }

    let mut debtors = Vec::new();

    // Per network, apply all or nothing
    let mut state_candidate = initial_state.clone();
    for (network, certificates) in certificate_by_network {
        let ret = state_candidate.apply_certificates_from(network, certificates);

        if let Err(ProofError::HasDebt { network }) = ret {
            debtors.push(network);
        }
    }

    if !debtors.is_empty() {
        return Err(ProofError::NotEnoughBalance { debtors });
    }

    Ok(state_candidate.get_checkpoint())
}
