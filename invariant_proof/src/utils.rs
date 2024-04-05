use tiny_keccak::{Hasher, Keccak};

pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut keccak256 = Keccak::v256();
    keccak256.update(data);
    let mut output = [0u8; 32];
    keccak256.finalize(&mut output);
    output
}
