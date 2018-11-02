use sha2::{Sha256, Digest};

use util::{Sha256Hash, LeafIndex, get_nth_bit};

pub struct ProducerTree {
    seed: Sha256Hash,
}

impl ProducerTree {
    pub fn new(seed: Sha256Hash) -> Self {
        Self { seed }
    }

    pub fn leaf(&self, index: LeafIndex) -> Sha256Hash {
        let mut value: [u8; 32] = self.seed.clone().into();
        for n in (0..63).rev() {
            if get_nth_bit(index.into(), n) {
                // flip bit
                let byte_number = n / 8;
                let bit_number = n % 8;
                value[byte_number] ^= 1 << bit_number;

                let mut hasher = Sha256::default();
                hasher.input(&value);
                value.copy_from_slice(&hasher.result());
            }
        }
        Sha256Hash::from(value)
    }
}

#[cfg(test)]
mod tests {
    use hex;

    use producer_tree::{ProducerTree, Sha256Hash};
    use util::LeafIndex;

    struct TestData<'a> {
        name:   &'a str,
        seed:   &'a str,
        index:  LeafIndex,
        output: &'a str,
    }

    const DERIVE_ELEMENT_TESTS: [TestData; 5] = [
        TestData {
            name:  "generate_from_seed 0 final node",
            seed:  "0000000000000000000000000000000000000000000000000000000000000000",
            index:  LeafIndex(0xffffffffffff),
            output: "02a40c85b6f28da08dfdbe0926c53fab2de6d28c10301f8f7c4073d5e42e3148",
        },
        TestData {
            name: "generate_from_seed FF final node",
            seed: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            index: LeafIndex(0xffffffffffff),
            output: "7cc854b54e3e0dcdb010d7a3fee464a9687be6e8db3be6854c475621e007a5dc",
        },
        TestData {
            name: "generate_from_seed FF alternate bits 1",
            index: LeafIndex(0xaaaaaaaaaaa),
            output: "56f4008fb007ca9acf0e15b054d5c9fd12ee06cea347914ddbaed70d1c13a528",
            seed: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        },
        TestData {
            name: "generate_from_seed FF alternate bits 2",
            index: LeafIndex(0x555555555555),
            output: "9015daaeb06dba4ccc05b91b2f73bd54405f2be9f217fbacd3c5ac2e62327d31",
            seed: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        },
        TestData {
            name: "generate_from_seed 01 last nontrivial node",
            index: LeafIndex(1),
            output: "915c75942a26bb3a433a8ce2cb0427c29ec6c1775cfc78328b57f6ba7bfeaa9c",
            seed: "0101010101010101010101010101010101010101010101010101010101010101",
        },
    ];

    #[test]
    fn test_specification_derive_element() {
        for test in &DERIVE_ELEMENT_TESTS {
            let mut seed = [0; 32];
            seed.copy_from_slice(&hex::decode(test.seed).unwrap());

            let producer = ProducerTree::new(Sha256Hash::from(seed));
            let leaf: [u8; 32] = producer.leaf(test.index).into();

            assert_eq!(hex::encode(leaf), test.output);
        }
    }
}