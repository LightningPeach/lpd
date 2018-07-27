use hex;

use element::{Element, Index, ROOT_INDEX};

struct TestData<'a> {
    name:   &'a str,
    seed:   &'a str,
	index:  Index,
	output: &'a str,
	// shouldFail bool
}

// DERIVE_ELEMENT_TESTS encodes the test vectors specified in BOLT-03,
// Appendix D, Generation Tests.
const DERIVE_ELEMENT_TESTS: [TestData; 5] = [
	TestData{
		name:   "generate_from_seed 0 final node",
		seed:   "0000000000000000000000000000000000000000000000000000000000000000",
		index:  Index(0xffffffffffff),
		output: "02a40c85b6f28da08dfdbe0926c53fab2de6d28c10301f8f7c4073d5e42e3148",
		// shouldFail: false,
	},
	TestData{
		name:   "generate_from_seed FF final node",
		seed:   "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
		index:  Index(0xffffffffffff),
		output: "7cc854b54e3e0dcdb010d7a3fee464a9687be6e8db3be6854c475621e007a5dc",
		// shouldFail: false,
	},
	TestData{
		name:   "generate_from_seed FF alternate bits 1",
		index:  Index(0xaaaaaaaaaaa),
		output: "56f4008fb007ca9acf0e15b054d5c9fd12ee06cea347914ddbaed70d1c13a528",
		seed:   "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
		// shouldFail: false,
	},
	TestData{
		name:   "generate_from_seed FF alternate bits 2",
		index:  Index(0x555555555555),
		output: "9015daaeb06dba4ccc05b91b2f73bd54405f2be9f217fbacd3c5ac2e62327d31",
		seed:   "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
		// shouldFail: false,
	},
	TestData{
		name:   "generate_from_seed 01 last nontrivial node",
		index:  Index(1),
		output: "915c75942a26bb3a433a8ce2cb0427c29ec6c1775cfc78328b57f6ba7bfeaa9c",
		seed:   "0101010101010101010101010101010101010101010101010101010101010101",
		// shouldFail: false,
	},
];

// test_specification_derive_element is used to check the consistency with
// specification hash derivation function.
#[test]
fn test_specification_derive_element() {
	for test in &DERIVE_ELEMENT_TESTS {
		// Generate seed element.
        let mut seed = [0; 32];
        seed.copy_from_slice(&hex::decode(test.seed).unwrap());

        let root_element = Element {
            index: ROOT_INDEX,
            hash:  seed,
        };

		// Derive element by index.
        let derived_element = root_element.derive(test.index).unwrap();

        // Generate element which we should get after derivation.
        let mut output = [0; 32];
        output.copy_from_slice(&hex::decode(test.output).unwrap());

        let output_element = Element{
            index: test.index,
            hash:  output,
        };

        // Check that they are equal.
        assert!(derived_element.is_equal(output_element));
	}
}