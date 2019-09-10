use dependencies::bitcoin_hashes;

use common_types::Sha256;

use bitcoin_hashes::Hash;
use super::util::{LeafIndex, get_nth_bit, count_trailing_zeroes, MAX_HEIGHT};
use super::error::{CanNotDeriveTreeElement, InvalidLeaf, CanNotFindElementByIndex, AddLeafError, LookupError};

use serde::{Serialize, Deserialize};

fn can_derive(from_index: LeafIndex, to_index: LeafIndex) -> bool {
    for bit in (count_trailing_zeroes(from_index.into())..63).rev() {
        if get_nth_bit(from_index.into(), bit) != get_nth_bit(to_index.into(), bit) {
            return false
        }
    }
    true
}

fn derive(from_index: LeafIndex, to_index: LeafIndex, from_value: Sha256) -> Result<Sha256, CanNotDeriveTreeElement> {
    if !can_derive(from_index, to_index) {
        return Err(CanNotDeriveTreeElement::new(from_index, to_index))
    }
    let mut value = from_value;
    for bit in 0..63 {
        if get_nth_bit(to_index.into(),bit) && !get_nth_bit(from_index.into(), bit) {
            // flip bit
            let byte_number = bit / 8;
            let bit_number = (bit % 8) as u8;
            value[byte_number] ^= 1 << bit_number;

//            let mut hasher = Sha256::default();
//            hasher.input(value.as_bytes());
            value = Sha256::hash(value.as_ref());
        }
    }
    return Ok(value)
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Leaf {
    index: LeafIndex,
    value: Sha256,
}

impl Leaf {
    fn new(index: LeafIndex, value: Sha256) -> Self {
        Self {
            index,
            value,
        }
    }

    pub fn get_index(&self) -> LeafIndex {
        self.index
    }

    pub fn get_value(&self) -> Sha256 {
        self.value
    }
}

pub struct StoreTree {
    known: [Leaf; MAX_HEIGHT],
    next_index: LeafIndex,
}

impl StoreTree {
    pub fn new() -> Self {
        Self {
            known: [Leaf::default(); MAX_HEIGHT],
            next_index: LeafIndex::new(0),
        }
    }

    pub fn add_leaf(&mut self, hash: Sha256) -> Result<(), AddLeafError> {
        {
            let next_index = self.next_index;
            self.receive_value(next_index, hash)?;
        }
        self.next_index.incr();
        Ok(())
    }

    pub fn lookup(&self, index: LeafIndex) -> Result<Sha256, LookupError> {
        for leaf in &self.known[..] {
            if can_derive(leaf.index, index) {
                let elem = derive(leaf.index, index, leaf.value)?;
                return Ok(elem);
            }
        }
        Err(CanNotFindElementByIndex::new(index).into())
    }

    fn receive_value(&mut self, index: LeafIndex, value: Sha256) -> Result<(), AddLeafError> {
        let pos = count_trailing_zeroes(index.into());
        // We should be able to generate every lesser value, otherwise invalid
        for i in 0..pos {
            if derive(index, self.known[i].index, value)? != self.known[i].value {
                return Err(InvalidLeaf::new(Leaf::new(index, value)).into())
            }
        }
        self.known[pos].index = index;
        self.known[pos].value = value;
        Ok(())
    }
}

#[cfg(test)]
impl Eq for StoreTree {}

#[cfg(test)]
impl PartialEq for StoreTree {
    fn eq(&self, other: &Self) -> bool {
        // TODO: compare them properly
        for i in 0..MAX_HEIGHT {
            if self.known[i] != other.known[i] {
                return false;
            }
        }
        self.next_index == other.next_index
    }
}

mod state_m {
    use super::StoreTree;

    use serde::{Serialize, Deserialize, Serializer, Deserializer};
    use state::DBValue;

    impl Serialize for StoreTree {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            use super::MAX_HEIGHT;
            use serde::ser::SerializeTuple;

            let mut tuple = serializer.serialize_tuple(MAX_HEIGHT + 1)?;
            self.known.iter().try_for_each(|leaf| tuple.serialize_element(leaf))?;
            tuple.serialize_element(&self.next_index)?;
            tuple.end()
        }
    }

    impl<'de> Deserialize<'de> for StoreTree {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            use super::MAX_HEIGHT;
            use std::fmt;
            use serde::de::{Visitor, SeqAccess, Error};

            struct V;

            impl<'r> Visitor<'r> for V {
                type Value = StoreTree;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'r>,
                {
                    let mut seq = seq;
                    let mut value = StoreTree::new();
                    value.known.iter_mut().try_for_each(|leaf| {
                        *leaf = seq.next_element()?.ok_or(Error::custom("expect known leaf"))?;
                        Ok(())
                    })?;
                    value.next_index = seq.next_element()?.ok_or(Error::custom("expect next_index"))?;
                    Ok(value)
                }
            }

            deserializer.deserialize_tuple(MAX_HEIGHT + 1, V)
        }
    }

    impl DBValue for StoreTree {
        type Extension = ();

        fn extend(self, e: Self::Extension) -> Self {
            let () = e;
            self
        }

        fn cf_name() -> &'static str {
            "store_tree"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StoreTree;
    use super::{LeafIndex, Sha256};

    struct TestInsert<'a> {
        index:      LeafIndex,
        secret:     &'a str,
        successful: bool,
    }

    struct TestData<'a> {
        name:    &'a str,
        inserts: &'a [TestInsert<'a>],
    }

    const TESTS: [TestData; 9] = [
        TestData{
            name:    "insert_secret correct sequence",
            inserts: &[
                TestInsert{
                    index:      LeafIndex(281474976710655),
                    secret:     "7cc854b54e3e0dcdb010d7a3fee464a9687be6e8db3be6854c475621e007a5dc",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710654),
                    secret:     "c7518c8ae4660ed02894df8976fa1a3659c1a8b4b5bec0c4b872abeba4cb8964",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710653),
                    secret:     "2273e227a5b7449b6e70f1fb4652864038b1cbf9cd7c043a7d6456b7fc275ad8",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710652),
                    secret:     "27cddaa5624534cb6cb9d7da077cf2b22ab21e9b506fd4998a51d54502e99116",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710651),
                    secret:     "c65716add7aa98ba7acb236352d665cab17345fe45b55fb879ff80e6bd0c41dd",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710650),
                    secret:     "969660042a28f32d9be17344e09374b379962d03db1574df5a8a5a47e19ce3f2",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710649),
                    secret:     "a5a64476122ca0925fb344bdc1854c1c0a59fc614298e50a33e331980a220f32",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710648),
                    secret:     "05cde6323d949933f7f7b78776bcc1ea6d9b31447732e3802e1f7ac44b650e17",
                    successful: true,
                },
            ],
        },
        TestData{
            name:    "insert_secret #1 incorrect",
            inserts: &[
                TestInsert{
                    index:      LeafIndex(281474976710655),
                    secret:     "02a40c85b6f28da08dfdbe0926c53fab2de6d28c10301f8f7c4073d5e42e3148",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710654),
                    secret:     "c7518c8ae4660ed02894df8976fa1a3659c1a8b4b5bec0c4b872abeba4cb8964",
                    successful: false,
                },
            ],
        },
        TestData{
            name:    "insert_secret #2 incorrect (#1 derived from incorrect)",
            inserts: &[
                TestInsert{
                    index:      LeafIndex(281474976710655),
                    secret:     "02a40c85b6f28da08dfdbe0926c53fab2de6d28c10301f8f7c4073d5e42e3148",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710654),
                    secret:     "dddc3a8d14fddf2b68fa8c7fbad2748274937479dd0f8930d5ebb4ab6bd866a3",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710653),
                    secret:     "2273e227a5b7449b6e70f1fb4652864038b1cbf9cd7c043a7d6456b7fc275ad8",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710652),
                    secret:     "27cddaa5624534cb6cb9d7da077cf2b22ab21e9b506fd4998a51d54502e99116",
                    successful: false,
                },
            ],
        },
        TestData{
            name:    "insert_secret #3 incorrect",
            inserts: &[
                TestInsert{
                    index:      LeafIndex(281474976710655),
                    secret:     "7cc854b54e3e0dcdb010d7a3fee464a9687be6e8db3be6854c475621e007a5dc",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710654),
                    secret:     "c7518c8ae4660ed02894df8976fa1a3659c1a8b4b5bec0c4b872abeba4cb8964",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710653),
                    secret:     "c51a18b13e8527e579ec56365482c62f180b7d5760b46e9477dae59e87ed423a",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710652),
                    secret:     "27cddaa5624534cb6cb9d7da077cf2b22ab21e9b506fd4998a51d54502e99116",
                    successful: false,
                },
            ],
        },
        TestData{
            name:    "insert_secret #4 incorrect (1,2,3 derived from incorrect)",
            inserts: &[
                TestInsert{
                    index:      LeafIndex(281474976710655),
                    secret:     "02a40c85b6f28da08dfdbe0926c53fab2de6d28c10301f8f7c4073d5e42e3148",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710654),
                    secret:     "dddc3a8d14fddf2b68fa8c7fbad2748274937479dd0f8930d5ebb4ab6bd866a3",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710653),
                    secret:     "c51a18b13e8527e579ec56365482c62f180b7d5760b46e9477dae59e87ed423a",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710652),
                    secret:     "ba65d7b0ef55a3ba300d4e87af29868f394f8f138d78a7011669c79b37b936f4",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710651),
                    secret:     "c65716add7aa98ba7acb236352d665cab17345fe45b55fb879ff80e6bd0c41dd",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710650),
                    secret:     "969660042a28f32d9be17344e09374b379962d03db1574df5a8a5a47e19ce3f2",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710649),
                    secret:     "a5a64476122ca0925fb344bdc1854c1c0a59fc614298e50a33e331980a220f32",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710649),
                    secret:     "05cde6323d949933f7f7b78776bcc1ea6d9b31447732e3802e1f7ac44b650e17",
                    successful: false,
                },
            ],
        },
        TestData{
            name:    "insert_secret #5 incorrect",
            inserts: &[
                TestInsert{
                    index:      LeafIndex(281474976710655),
                    secret:     "7cc854b54e3e0dcdb010d7a3fee464a9687be6e8db3be6854c475621e007a5dc",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710654),
                    secret:     "c7518c8ae4660ed02894df8976fa1a3659c1a8b4b5bec0c4b872abeba4cb8964",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710653),
                    secret:     "2273e227a5b7449b6e70f1fb4652864038b1cbf9cd7c043a7d6456b7fc275ad8",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710652),
                    secret:     "27cddaa5624534cb6cb9d7da077cf2b22ab21e9b506fd4998a51d54502e99116",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710651),
                    secret:     "631373ad5f9ef654bb3dade742d09504c567edd24320d2fcd68e3cc47e2ff6a6",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710650),
                    secret:     "969660042a28f32d9be17344e09374b379962d03db1574df5a8a5a47e19ce3f2",
                    successful: false,
                },
            ],
        },
        TestData{
            name:    "insert_secret #6 incorrect (5 derived from incorrect)",
            inserts: &[
                TestInsert{
                    index:      LeafIndex(281474976710655),
                    secret:     "7cc854b54e3e0dcdb010d7a3fee464a9687be6e8db3be6854c475621e007a5dc",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710654),
                    secret:     "c7518c8ae4660ed02894df8976fa1a3659c1a8b4b5bec0c4b872abeba4cb8964",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710653),
                    secret:     "2273e227a5b7449b6e70f1fb4652864038b1cbf9cd7c043a7d6456b7fc275ad8",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710652),
                    secret:     "27cddaa5624534cb6cb9d7da077cf2b22ab21e9b506fd4998a51d54502e99116",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710651),
                    secret:     "631373ad5f9ef654bb3dade742d09504c567edd24320d2fcd68e3cc47e2ff6a6",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710650),
                    secret:     "b7e76a83668bde38b373970155c868a653304308f9896692f904a23731224bb1",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710649),
                    secret:     "a5a64476122ca0925fb344bdc1854c1c0a59fc614298e50a33e331980a220f32",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710648),
                    secret:     "05cde6323d949933f7f7b78776bcc1ea6d9b31447732e3802e1f7ac44b650e17",
                    successful: false,
                },
            ],
        },
        TestData{
            name:    "insert_secret #7 incorrect",
            inserts: &[
                TestInsert{
                    index:      LeafIndex(281474976710655),
                    secret:     "7cc854b54e3e0dcdb010d7a3fee464a9687be6e8db3be6854c475621e007a5dc",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710654),
                    secret:     "c7518c8ae4660ed02894df8976fa1a3659c1a8b4b5bec0c4b872abeba4cb8964",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710653),
                    secret:     "2273e227a5b7449b6e70f1fb4652864038b1cbf9cd7c043a7d6456b7fc275ad8",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710652),
                    secret:     "27cddaa5624534cb6cb9d7da077cf2b22ab21e9b506fd4998a51d54502e99116",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710651),
                    secret:     "c65716add7aa98ba7acb236352d665cab17345fe45b55fb879ff80e6bd0c41dd",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710650),
                    secret:     "969660042a28f32d9be17344e09374b379962d03db1574df5a8a5a47e19ce3f2",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710649),
                    secret:     "e7971de736e01da8ed58b94c2fc216cb1dca9e326f3a96e7194fe8ea8af6c0a3",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710648),
                    secret:     "05cde6323d949933f7f7b78776bcc1ea6d9b31447732e3802e1f7ac44b650e17",
                    successful: false,
                },
            ],
        },
        TestData{
            name: "insert_secret #8 incorrect",
            inserts: &[
                TestInsert{
                    index:      LeafIndex(281474976710655),
                    secret:     "7cc854b54e3e0dcdb010d7a3fee464a9687be6e8db3be6854c475621e007a5dc",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710654),
                    secret:     "c7518c8ae4660ed02894df8976fa1a3659c1a8b4b5bec0c4b872abeba4cb8964",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710653),
                    secret:     "2273e227a5b7449b6e70f1fb4652864038b1cbf9cd7c043a7d6456b7fc275ad8",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710652),
                    secret:     "27cddaa5624534cb6cb9d7da077cf2b22ab21e9b506fd4998a51d54502e99116",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710651),
                    secret:     "c65716add7aa98ba7acb236352d665cab17345fe45b55fb879ff80e6bd0c41dd",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710650),
                    secret:     "969660042a28f32d9be17344e09374b379962d03db1574df5a8a5a47e19ce3f2",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710649),
                    secret:     "a5a64476122ca0925fb344bdc1854c1c0a59fc614298e50a33e331980a220f32",
                    successful: true,
                },
                TestInsert{
                    index:      LeafIndex(281474976710648),
                    secret:     "a7efbc61aac46d34f77778bac22c8a20c6a46ca460addc49009bda875ec88fa4",
                    successful: false,
                },
            ],
        },
    ];

    #[test]
    fn test_specification_sha_chain_insert() {
        for test in &TESTS {
            let mut receiver = StoreTree::new();
            for insert in test.inserts {
                let secret = Sha256::from_hex(insert.secret).unwrap();
                let resp = receiver.add_leaf(secret);
                if resp.is_err() && insert.successful {
                    panic!("Failed ({}): error was received but it shouldn't: {}", test.name, resp.unwrap_err())
                } else if resp.is_ok() && !insert.successful {
                    panic!("Failed ({}): error wasn't received", test.name)
                }
            }
        }
    }

    #[test]
    fn test_db() {
        use state::DBBuilder;
        use std::{fs, io};

        fn test_trees() -> Vec<StoreTree> {
            TESTS.iter().map(|test| {
                let mut receiver = StoreTree::new();
                for insert in test.inserts {
                    if insert.successful {
                        let secret = Sha256::from_hex(insert.secret).unwrap();
                        receiver.add_leaf(secret).unwrap();
                    } else {
                        break
                    }
                }
                receiver
            }).collect()
        }

        {
            let () = fs::remove_dir_all("target/db")
                .or_else(|e| if e.kind() == io::ErrorKind::NotFound { Ok(()) } else { Err(e) })
                .unwrap();
            let db = DBBuilder::default().register::<StoreTree>().build("target/db").unwrap();
            for (index, tree) in test_trees().into_iter().enumerate() {
                db.put(&index, tree).ok().unwrap();
            }
        }

        let db = DBBuilder::default().register::<StoreTree>().build("target/db").unwrap();
        for (index, tree) in test_trees().into_iter().enumerate() {
            let from_db: StoreTree = db.get(&index).unwrap().unwrap();
            if from_db != tree {
                panic!();
            }
        }
    }
}
