#![no_main]

use poly_pessimistic_proof::{certificate::Certificate, generate_full_proof_with_state, State};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let initial_state = sp1_zkvm::io::read::<State>();
    let certificates = sp1_zkvm::io::read::<Vec<Certificate>>();

    let new_roots = generate_full_proof_with_state(initial_state, certificates).unwrap();

    sp1_zkvm::io::commit(&new_roots);
}
