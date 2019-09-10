use super::element::{Index, MAX_HEIGHT};
use std;

// get_bit return bit on index at position.
pub fn get_bit(index: Index, position: u8) -> u8 {
	((index.0 >> (position as u64)) & 1) as u8
}

pub fn get_prefix(index: Index, position: u8) -> u64 {
	//	+ -------------------------- +
	// 	| №  | value | mask | return |
	//	+ -- + ----- + ---- + ------ +
	//	| 63 |	 1   |  0   |	 0   |
	//	| 62 |	 0   |  0   |	 0   |
	//	| 61 |   1   |  0   |	 0   |
	//		....
	//	|  4 |	 1   |  0   |	 0   |
	//	|  3 |   1   |  0   |	 0   |
	//	|  2 |   1   |  1   |	 1   | <--- position
	//	|  1 |   0   |  1   |	 0   |
	//	|  0 |   1   |  1   |	 1   |
	//	+ -- + ----- + ---- + ------ +

    let mask = std::u64::MAX - ((1u64 << (position as u64)) - 1);
    index.0 & mask
}

// count_trailing_zeros counts number of trailing zero bits, this function is
// used to determine the number of element bucket.
pub fn count_trailing_zeros(index: Index) -> u8 {
	for zeros in 0..MAX_HEIGHT {
		if get_bit(index, zeros) != 0 {
			return zeros
		}
	}
	MAX_HEIGHT
}
