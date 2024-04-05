use super::Hasher;
use tiny_keccak::{Hasher as _, Keccak};

pub struct Keccak256;

impl Hasher for Keccak256 {
    type Digest = [u8; 32];

    fn merge(left: &[u8; 32], right: &[u8; 32]) -> Self::Digest {
        let mut keccak256 = Keccak::v256();
        keccak256.update(left);
        keccak256.update(right);

        let mut output = [0u8; 32];
        keccak256.finalize(&mut output);
        output
    }
}
