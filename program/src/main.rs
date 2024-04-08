#![no_main]

use poly_invariant_proof::{
    keccak::KeccakDigest,
    leaf_proof,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    Withdrawal,
};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let local_exit_tree = sp1_zkvm::io::read::<LocalExitTree<Keccak256Hasher>>();
    let local_exit_root = sp1_zkvm::io::read::<KeccakDigest>();
    let withdrawals = sp1_zkvm::io::read::<Vec<Withdrawal>>();

    let (new_local_exit_root, aggregate_deposits) =
        leaf_proof(local_exit_tree, local_exit_root, withdrawals).unwrap();

    sp1_zkvm::io::commit(&new_local_exit_root);
    sp1_zkvm::io::commit(&aggregate_deposits.hash());
}
