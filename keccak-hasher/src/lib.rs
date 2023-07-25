use hash256_std_hasher::Hash256StdHasher;
use hash_db::Hasher;
use tiny_keccak::{Hasher as _, Keccak};

/// The `Keccak` hash output type.
pub type KeccakHash = [u8; 32];

struct KeccakHasher;

impl Hasher for KeccakHasher {
    type Out = KeccakHash;

    fn hash(data: &[u8]) -> Self::Out {
        let mut output = [0u8; 32];
        let mut keccak = Keccak::v256();
        keccak.update(data);
        keccak.finalize(&mut output);
        output
    }

    type StdHasher = Hash256StdHasher;

    const LENGTH: usize = 32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn hash256_std_hasher_works() {
        let hello_bytes = b"Hello world!";
        let hello_key = KeccakHasher::hash(hello_bytes);

        let mut h: HashMap<<KeccakHasher as Hasher>::Out, Vec<u8>> = Default::default();
        h.insert(hello_key, hello_bytes.to_vec());
        h.remove(&hello_key);

        let mut h: HashMap<
            <KeccakHasher as Hasher>::Out,
            Vec<u8>,
            std::hash::BuildHasherDefault<Hash256StdHasher>,
        > = Default::default();
        h.insert(hello_key, hello_bytes.to_vec());
        h.remove(&hello_key);
    }
}
