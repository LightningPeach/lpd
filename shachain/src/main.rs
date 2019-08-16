use dependencies::sha2;
use dependencies::hex;

use sha2::{Sha256, Digest};

use producer::RevocationProducer;
use element::{Element, Index, ROOT_INDEX};

mod test_producer;
mod test_store;
mod test_element;

mod producer;
mod store;
mod element;
mod utils;

//name: generate_from_seed 0 final node
//seed: 0x0000000000000000000000000000000000000000000000000000000000000000
//I: 281474976710655
//output: 0x02a40c85b6f28da08dfdbe0926c53fab2de6d28c10301f8f7c4073d5e42e3148

fn main() {
//    let seed_str = "0000000000000000000000000000000000000000000000000000000000000000";
//    let mut seed = [0; 32];
//    seed.copy_from_slice(&hex::decode(seed_str).unwrap());
//
//    let producer = RevocationProducer::new(seed);
//    let hash = producer.at_index(0).unwrap();
//    let hash_str = hex::encode(&hash);
//    println!("{}", hash_str);
//
//    let root_element = Element{
//        index: ROOT_INDEX,
//        hash:  seed,
//    };
//
//    let derived_element = root_element.derive(Index(281474976710655)).unwrap();
//    let hash_str = hex::encode(&derived_element.hash);
//    println!("{}", hash_str);

    let seed_str = "0000000000000000000000000000000000000000000000000000000000000000";
    let mut seed = [0; 32];
    seed.copy_from_slice(&hex::decode(seed_str).unwrap());

    let root_element = Element {
        index: ROOT_INDEX,
        hash:  seed,
    };

    let derived_element = root_element.derive(Index(281474976710655)).unwrap();
    let hash_str = hex::encode(&derived_element.hash);
    println!("{}", hash_str);


//    let seed_str = "0101010101010101010101010101010101010101010101010101010101010101";
//    let mut seed = [0; 32];
//    seed.copy_from_slice(&hex::decode(seed_str).unwrap());

//    let position =  0;
//    let byte_number = position / 8;
//    let bit_number = position % 8;
//    seed[byte_number] ^= (1 << bit_number);

//    let mut hasher = Sha256::default();
//    hasher.input(&seed);
//    let hash = hasher.result();
//    let hash_str = hex::encode(&hash);
//    println!("{}", hash_str);

//    let root_element = Element {
//        index: ROOT_INDEX,
//        hash:  seed,
//    };
//
//    let derived_element = root_element.derive(Index(1)).unwrap();
//    let hash_str = hex::encode(&derived_element.hash);
//    println!("{}", hash_str);
}
