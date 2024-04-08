use tiny_keccak::{Hasher as _, Keccak};

use crate::keccak::KeccakDigest;

/// A hasher used in constructing a [`super::LocalExitTree`].
pub trait Hasher {
    type Digest;

    /// Hashes two digests into one.
    fn merge(left: &Self::Digest, right: &Self::Digest) -> Self::Digest;
}

/// A Keccak hasher with a 256-bit security level.
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
