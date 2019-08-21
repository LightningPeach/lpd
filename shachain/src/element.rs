use dependencies::hex;
use dependencies::bitcoin_hashes;

use std::error::Error;
use common_types::Sha256;
use bitcoin_hashes::Hash;

use crate::utils;

// element represents the entity which contains the hash and index
// corresponding to it. An element is the output of the shachain PRF. By
// comparing two indexes we're able to mutate the hash in such way to derive
// another element.
#[derive(Copy, Clone, Default)]
pub struct Element {
	pub index: Index,
	pub hash:  [u8; 32],
}

// newElementFromStr creates new element from the given hash string.
//func newElementFromStr(s string, index index) (*element, error) {
//	hash, err := hashFromString(s)
//	if err != nil {
//		return nil, err
//	}
//
//	return &element{
//		index: index,
//		hash:  *hash,
//	}, nil
//}

impl Element {
    // derive computes one shachain element from another by applying a series of
    // bit flips and hashing operations based on the starting and ending index.
    pub fn derive(&self, to_index: Index) -> Result<Element, Box<Error>> {
        let positions= self.index.derive_bit_transformations(to_index)?;

        let mut hash = self.hash;
        for position in positions {
            // Flip the bit and then hash the current state.
            let byte_number = position / 8;
            let bit_number = position % 8;

            hash[byte_number as usize] ^= (1 << bit_number);

            hash = Sha256::hash(hash.as_ref()).into_inner();
        }

        Ok(Element {
            index: to_index,
            hash,
        })
    }

    // is_equal returns true if two elements are identical and false otherwise.
    pub fn is_equal(&self, other: Element) -> bool {
        return self.index.0 == other.index.0 && self.hash == other.hash
    }
}

// MAX_HEIGHT is used to determine the maximum allowable index and the
// length of the array required to order to derive all previous hashes
// by index. The entries of this array as also known as buckets.
pub const MAX_HEIGHT: u8 = 48;

// ROOT_INDEX is an index which corresponds to the root hash.
pub const ROOT_INDEX: Index = Index(0);

// START_INDEX is the index of first element in the shachain PRF.
pub const START_INDEX: Index = Index((1 << MAX_HEIGHT) - 1);

// Index is a number which identifies the hash number and serves as a way to
// determine the hashing operation required  to derive one hash from another.
// index is initialized with the startIndex and decreases down to zero with
// successive derivations.
#[derive(Copy, Clone, Default)]
pub struct Index(pub u64);

impl Index {
    // new is used to create index instance. The inner operations with index
    // implies that index decreasing from some max number to zero, but for
    // simplicity and backward compatibility with previous logic it was transformed
    // to work in opposite way.
    pub fn new(v: u64) -> Self {
        Index(START_INDEX.0 - v)
    }

    // derive_bit_transformations function checks that the 'to' index is derivable
    // from the 'from' index by checking the indexes are prefixes of another. The
    // bit positions where the zeroes should be changed to ones in order for the
    // indexes to become the same are returned. This set of bits is needed in order
    // to derive one hash from another.
    //
    // NOTE: The index 'to' is derivable from index 'from' iff index 'from' lies
    // left and above index 'to' on graph below, for example:
    // 1. 7(0b111) -> 7
    // 2. 6(0b110) -> 6,7
    // 3. 5(0b101) -> 5
    // 4. 4(0b100) -> 4,5,6,7
    // 5. 3(0b011) -> 3
    // 6. 2(0b010) -> 2, 3
    // 7. 1(0b001) -> 1
    //
    //    ^ bucket number
    //    |
    //  3 |   x
    //    |   |
    //  2 |   |               x
    //    |   |               |
    //  1 |   |       x       |       x
    //    |   |       |       |       |
    //  0 |   |   x   |   x   |   x   |   x
    //    |   |   |   |   |   |   |   |   |
    //    +---|---|---|---|---|---|---|---|---> index
    //        0   1   2   3   4   5   6   7
    //
    fn derive_bit_transformations(&self, to: Index) -> Result<Vec<u8>, Box<Error>> {
        let mut positions = Vec::new();

        if self.0 == to.0 {
            return Ok(positions)
        }

        //	+ --------------- +
        // 	| №  | from | to  |
        //	+ -- + ---- + --- +
        //	| 48 |	 1  |  1  |
        //	| 47 |	 0  |  0  | [48-5] - same part of 'from' and 'to'
        //	| 46 |   0  |  0  |	    indexes which also is called prefix.
        //		....
        //	|  5 |	 1  |  1  |
        //	|  4 |	 0  |  1  | <--- position after which indexes becomes
        //	|  3 |   0  |  0  |	 different, after this position
        //	|  2 |   0  |  1  |	 bits in 'from' index all should be
        //	|  1 |   0  |  0  |	 zeros or such indexes considered to be
        //	|  0 |   0  |  1  |	 not derivable.
        //	+ -- + ---- + --- +
        let zeros = utils::count_trailing_zeros(*self);
        if self.0 != utils::get_prefix(to, zeros) {
            return Err(From::from("prefixes are different - indexes aren't derivable"));
        }

        // The remaining part of 'to' index represents the positions which we
        // will use then in order to derive one element from another.
        for position in (0..zeros).rev() {
            if utils::get_bit(to, position) == 1 {
                positions.push(position);
            }
        }

        Ok(positions)
    }
}