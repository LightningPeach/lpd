use std::error::Error;

use element::{Element, Index, MAX_HEIGHT, START_INDEX};
use utils;

// RevocationStore is a concrete implementation of the Store interface. The
// revocation store is able to efficiently store N derived shachain elements in
// a space efficient manner with a space complexity of O(log N). The original
// description of the storage methodology can be found here:
// https://github.com/lightningnetwork/lightning-rfc/blob/master/03-transactions.md#efficient-per-commitment-secret-storage
pub struct RevocationStore {
	// len_buckets stores the number of currently active buckets.
	len_buckets: u8,

	// buckets is an array of elements from which we may derive all
	// previous elements, each bucket corresponds to the element with the
	// particular number of trailing zeros.
	buckets: [Element; MAX_HEIGHT as usize],

	// index is an available index which will be assigned to the new
	// element.
	index: Index,
}

impl RevocationStore {
    // new creates the new shachain store.
    pub fn new() -> Self {
    	Self {
    		len_buckets: 0,
            buckets:    [Element::default(); MAX_HEIGHT as usize],
    		index:      START_INDEX,
    	}
    }

    // look_up function is used to restore/lookup/fetch the previous secret by its
    // index. If secret which corresponds to given index was not previously placed
    // in store we will not able to derive it and function will fail.
    //
    // NOTE: This function is part of the Store interface.
    fn look_up(&self, v: u64) -> Result<[u8; 32], Box<Error>> {
    	let ind = Index::new(v);

    	// Trying to derive the index from one of the existing buckets elements.
        for i in 0..self.len_buckets as usize {
            if let Ok(element) = self.buckets[i].derive(ind) {
                return Ok(element.hash)
            }
        }
        Err(From::from(format!("unable to derive hash #{}", ind.0)))
    }

    // add_next_entry attempts to store the given hash within its internal storage in
    // an efficient manner.
    //
    // NOTE: The hashes derived from the shachain MUST be inserted in the order
    // they're produced by a shachain.Producer.
    //
    // NOTE: This function is part of the Store interface.
    pub fn add_next_entry(&mut self, hash: [u8; 32]) -> Result<(), Box<Error>> {
    	let new_element = Element{
    		index: self.index,
    		hash,
    	};

        let bucket = utils::count_trailing_zeros(new_element.index);

        for i in 0..bucket {
            let e = new_element.derive(self.buckets[i as usize].index)?;

            if !e.is_equal(self.buckets[i as usize]) {
                return Err(From::from("hash isn't derivable from previous ones"));
            }
        }

    	self.buckets[bucket as usize] = new_element;
    	if bucket + 1 > self.len_buckets {
    		self.len_buckets = bucket + 1
    	}

    	self.index.0 -= 1;
        Ok(())
    }
}