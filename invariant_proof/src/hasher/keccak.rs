use tiny_keccak::{Hasher as _, Keccak};

use super::Hasher;

pub type KeccakDigest = [u8; 32];

pub fn keccak256(data: &[u8]) -> KeccakDigest {
    let mut hasher = Keccak::v256();
    hasher.update(data);

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

pub struct Keccak256Hasher;

impl Hasher for Keccak256Hasher {
    type Digest = KeccakDigest;

    fn merge(left: &KeccakDigest, right: &KeccakDigest) -> KeccakDigest {
        let mut keccak256 = Keccak::v256();
        keccak256.update(left);
        keccak256.update(right);

        let mut output = [0u8; 32];
        keccak256.finalize(&mut output);
        output
    }
}
