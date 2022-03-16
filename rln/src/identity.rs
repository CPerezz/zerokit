// Adapted from
// https://github.com/worldcoin/semaphore-rs/blob/main/src/identity.rs

use num_bigint::{BigInt, Sign};
use once_cell::sync::Lazy;
use poseidon_rs::Poseidon;
use sha2::{Digest, Sha256};

use crate::util::{bigint_to_fr, fr_to_bigint};

static POSEIDON: Lazy<Poseidon> = Lazy::new(Poseidon::new);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Identity {
    pub trapdoor:  BigInt,
    pub nullifier: BigInt,
}

// todo: improve
fn sha(msg: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(msg);
    let result = hasher.finalize();
    let res: [u8; 32] = result.into();
    res
}

impl Identity {
    pub fn new(seed: &[u8]) -> Self {
        let seed_hash = &sha(seed);

        // https://github.com/appliedzkp/zk-kit/blob/1ea410456fc2b95877efa7c671bc390ffbfb5d36/packages/identity/src/identity.ts#L58
        let trapdoor = BigInt::from_bytes_be(
            Sign::Plus,
            &sha(format!("{}identity_trapdoor", hex::encode(seed_hash)).as_bytes()),
        );
        let nullifier = BigInt::from_bytes_be(
            Sign::Plus,
            &sha(format!("{}identity_nullifier", hex::encode(seed_hash)).as_bytes()),
        );

        Self {
            trapdoor,
            nullifier,
        }
    }

    pub fn secret_hash(&self) -> BigInt {
        let res = POSEIDON
            .hash(vec![
                bigint_to_fr(&self.nullifier),
                bigint_to_fr(&self.trapdoor),
            ])
            .unwrap();
        fr_to_bigint(res)
    }

    pub fn commitment(&self) -> BigInt {
        let res = POSEIDON
            .hash(vec![bigint_to_fr(&self.secret_hash())])
            .unwrap();
        fr_to_bigint(res)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_identity() {
        let id = Identity::new(b"message");
        //println!("Commitment: {:#}", id.commitment());
        // From https://github.com/appliedzkp/zk-kit/blob/1ea410456fc2b95877efa7c671bc390ffbfb5d36/packages/identity/tests/index.test.ts#L63-L70
        let s = "1720349790382552497189398984241859233944354304766757200361065203741879866188";

        let x = BigInt::from_str(s).unwrap();

        assert!(id.commitment() == x);

    }
}