use std::error::Error;

use element::{Element, Index, ROOT_INDEX};

// RevocationProducer is an implementation of Producer interface using the
// shachain PRF construct. Starting with a single 32-byte element generated
// from a CSPRNG, shachain is able to efficiently generate a nearly unbounded
// number of secrets while maintaining a constant amount of storage. The
// original description of shachain can be found here:
// https://github.com/rustyrussell/ccan/blob/master/ccan/crypto/shachain/design.txt
// with supplementary material here:
// https://github.com/lightningnetwork/lightning-rfc/blob/master/03-transactions.md#per-commitment-secret-requirements
pub struct RevocationProducer {
	// root is the element from which we may generate all hashes which
	// corresponds to the index domain [281474976710655,0].
	root: Element,
}

impl RevocationProducer {
    // new creates new instance of shachain producer.
    pub fn new(root: [u8; 32]) -> Self {
        Self {
            root: Element {
    			index: ROOT_INDEX,
    			hash:  root,
    		},
        }
    }

    // at_index produces a secret by evaluating using the initial seed and a
    // particular index.
    //
    // NOTE: Part of the Producer interface.
    pub fn at_index(&self, v: u64) -> Result<[u8; 32], Box<Error>> {
    	let ind = Index::new(v);
        let element = self.root.derive(ind)?;
        Ok(element.hash)
    }
}