use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
use bitcoin_hashes::Hash;

// Represent reordering of transaction input and outputs
// inputs is a vector of positions of initial transaction inputs
// for example
// inputs = [2, 0, 1] means that in a new transaction inputs:
// 0: 2-nd initial
// 1: 0-th initial
// 2: 1-st initial
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TransactionReordering {
    pub inputs: Vec<u32>,
    pub outputs: Vec<u32>,
}


fn reverse_u8_32(mut x: [u8; 32]) -> [u8; 32] {
    for i in 0..15 {
        let tmp = x[i];
        x[i]= x[31-i];
        x[31-i] = tmp;
    }
    x
}

impl TransactionReordering {
    // Applies reordering to a given transaction
    // Returns true if reordering was successful
    pub fn apply(&self, tx: &mut Transaction) -> bool {
        if !(
            self.is_correct()
            && self.inputs.len()==tx.input.len()
            && self.outputs.len()==tx.output.len()
        ) {
            return false;
        }

        let mut new_inputs = Vec::<TxIn>::with_capacity(tx.input.len());
        for i in 0..tx.input.len() {
            let x = self.inputs[i] as usize;
            new_inputs.push(tx.input[x].clone());
        }

        let mut new_outputs = Vec::<TxOut>::with_capacity(tx.output.len());
        for i in 0..tx.output.len() {
            let x = self.outputs[i] as usize;
            new_outputs.push(tx.output[x].clone());
        }

        tx.input = new_inputs;
        tx.output = new_outputs;
        return true;
    }

    // Returns if reordering is correct
    // inputs are some permutation of 0..len(inputs)
    // outputs are some permutation of 0..len(outputs)
    pub fn is_correct(&self) -> bool {
        return is_permutation(&self.inputs) && is_permutation(&self.outputs);
    }

    // returns reordering which applied to transaction makes it bip69
    pub fn from_tx_to_bip69(tx: &Transaction) -> TransactionReordering {
        let mut inp_ind: Vec<u32> = (0u32..tx.input.len() as u32).collect();
        let mut out_ind: Vec<u32> = (0u32..tx.output.len() as u32).collect();

        inp_ind.sort_unstable_by(|i, j| {
            let h1 = reverse_u8_32(tx.input[*i as usize].previous_output.txid.into_inner());
            let h2 = reverse_u8_32(tx.input[*j as usize].previous_output.txid.into_inner());
            let prev_hash_ordering = h1.cmp(&h2);
            let index_ordering = tx.input[*i as usize].previous_output.vout.cmp(&tx.input[*j as usize].previous_output.vout);
            return prev_hash_ordering.then(index_ordering);
        });
        out_ind.sort_unstable_by(|i, j|{
            let amount_ordering = tx.output[*i as usize].value.cmp(&tx.output[*j as usize].value);
            let sc1 = tx.output[*i as usize].script_pubkey.as_bytes();
            let sc2 = tx.output[*j as usize].script_pubkey.as_bytes();
            let script_ordering = sc1.cmp(&sc2);
            amount_ordering.then(script_ordering)
        });
        return TransactionReordering {
            inputs: inp_ind,
            outputs: out_ind,
        }
    }
}

// checks if numbers are permutation of 0..len(v)
fn is_permutation(v: &Vec<u32>) -> bool {
    let mut a = v.clone();
    a.sort_unstable();
    for i in 0..a.len() {
        if a[i] != (i as u32) {
            return false;
        }
    }
    return true;
}

// Reorder inputs and outputs of the transaction. Ordering is defined in BIP69
pub fn reorder_tx(tx: &mut Transaction) -> TransactionReordering {
    let reordering = TransactionReordering::from_tx_to_bip69(tx);
    reordering.apply(tx);
    return reordering;
}

#[cfg(test)]
mod tests {
    use super::{is_permutation, TransactionReordering, reorder_tx};
    use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
    use bitcoin::blockdata::script::Script;
    use bitcoin::OutPoint;
    use super::super::tools::{s2script, s2dh256};

    fn get_bip69_ex1() -> Transaction {
        let tx = Transaction {
            version: 1,
            input: vec![
                TxIn {
                    previous_output: OutPoint{
                        txid: s2dh256("35288d269cee1941eaebb2ea85e32b42cdb2b04284a56d8b14dcc3f5c65d6055"),
                        vout: 0,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("35288d269cee1941eaebb2ea85e32b42cdb2b04284a56d8b14dcc3f5c65d6055"),
                        vout: 1,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
            ],
            output: vec![
                TxOut {
                    value: 100000000,
                    script_pubkey: s2script("41046a0765b5865641ce08dd39690aade26dfbf5511430ca428a3089261361cef170e3929a68aee3d8d4848b0c5111b0a37b82b86ad559fd2a745b44d8e8d9dfdc0cac"),
                },
                TxOut {
                    value: 2400000000,
                    script_pubkey: s2script("41044a656f065871a353f216ca26cef8dde2f03e8c16202d2e8ad769f02032cb86a5eb5e56842e92e19141d60a01928f8dd2c875a390f67c1f6c94cfc617c0ea45afac")
                },
            ],
            lock_time: 0,
        };
        return tx;
    }

    fn get_bip69_ex2() -> Transaction {
        let tx = Transaction {
            version: 1,
            input: vec![
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("0e53ec5dfb2cb8a71fec32dc9a634a35b7e24799295ddd5278217822e0b31f57"),
                        vout: 0,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("26aa6e6d8b9e49bb0630aac301db6757c02e3619feb4ee0eea81eb1672947024"),
                        vout: 1,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("28e0fdd185542f2c6ea19030b0796051e7772b6026dd5ddccd7a2f93b73e6fc2"),
                        vout: 0,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("381de9b9ae1a94d9c17f6a08ef9d341a5ce29e2e60c36a52d333ff6203e58d5d"),
                        vout: 1,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("3b8b2f8efceb60ba78ca8bba206a137f14cb5ea4035e761ee204302d46b98de2"),
                        vout: 0,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("402b2c02411720bf409eff60d05adad684f135838962823f3614cc657dd7bc0a"),
                        vout: 1,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("54ffff182965ed0957dba1239c27164ace5a73c9b62a660c74b7b7f15ff61e7a"),
                        vout: 1,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("643e5f4e66373a57251fb173151e838ccd27d279aca882997e005016bb53d5aa"),
                        vout: 0,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("6c1d56f31b2de4bfc6aaea28396b333102b1f600da9c6d6149e96ca43f1102b1"),
                        vout: 1
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("7a1de137cbafb5c70405455c49c5104ca3057a1f1243e6563bb9245c9c88c191"),
                        vout: 0,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("7d037ceb2ee0dc03e82f17be7935d238b35d1deabf953a892a4507bfbeeb3ba4"),
                        vout: 1,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("a5e899dddb28776ea9ddac0a502316d53a4a3fca607c72f66c470e0412e34086"),
                        vout: 0,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("b4112b8f900a7ca0c8b0e7c4dfad35c6be5f6be46b3458974988e1cdb2fa61b8"),
                        vout: 0,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("bafd65e3c7f3f9fdfdc1ddb026131b278c3be1af90a4a6ffa78c4658f9ec0c85"),
                        vout: 0
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("de0411a1e97484a2804ff1dbde260ac19de841bebad1880c782941aca883b4e9"),
                        vout: 1
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("f0a130a84912d03c1d284974f563c5949ac13f8342b8112edff52971599e6a45"),
                        vout: 0,
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: s2dh256("f320832a9d2e2452af63154bc687493484a0e7745ebd3aaf9ca19eb80834ad60"),
                        vout: 0
                    },
                    script_sig: Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                },
            ],
            output: vec![
                TxOut {
                    value: 400057456,
                    script_pubkey: s2script("76a9144a5fba237213a062f6f57978f796390bdcf8d01588ac"),
                },
                TxOut {
                    value: 40000000000,
                    script_pubkey: s2script("76a9145be32612930b8323add2212a4ec03c1562084f8488ac")
                },
            ],
            lock_time: 0,
        };
        return tx;
    }

    #[test]
    fn test_is_permutation() {
        assert_eq!(is_permutation(&vec![0, 1, 2]), true);
        assert_eq!(is_permutation(&vec![1, 2, 3]), false);
        assert_eq!(is_permutation(&vec![3, 1, 2, 0]), true);
        assert_eq!(is_permutation(&vec![0, 1, 1, 2]), false);
    }

    #[test]
    fn test_from_tx_to_bip69_1(){
        let mut tx = get_bip69_ex1();
        assert_eq!(TransactionReordering::from_tx_to_bip69(&tx), TransactionReordering{
            inputs: vec![0, 1],
            outputs: vec![0, 1],
        });
        TransactionReordering{
            inputs: vec![1, 0],
            outputs: vec![1, 0],
        }.apply(&mut tx);
        assert_eq!(TransactionReordering::from_tx_to_bip69(&tx), TransactionReordering{
            inputs: vec![1, 0],
            outputs: vec![1, 0],
        });
    }

    #[test]
    fn test_from_tx_to_bip69_2(){
        let mut tx = get_bip69_ex2();
        assert_eq!(TransactionReordering::from_tx_to_bip69(&tx), TransactionReordering{
            inputs: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            outputs: vec![0, 1],
        });
        TransactionReordering{
            inputs: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 0],
            outputs: vec![0, 1],
        }.apply(&mut tx);
        assert_eq!(TransactionReordering::from_tx_to_bip69(&tx), TransactionReordering{
            inputs: vec![16, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            outputs: vec![0, 1],
        });
    }

    #[test]
    fn test_reorder_tx_1() {
        let mut tx = get_bip69_ex1();
        assert_eq!(reorder_tx(&mut tx), TransactionReordering{
            inputs: vec![0, 1],
            outputs: vec![0, 1],
        });
        assert_eq!(tx, get_bip69_ex1());

        TransactionReordering{
            inputs: vec![1, 0],
            outputs: vec![1, 0],
        }.apply(&mut tx);

        assert_eq!(reorder_tx(&mut tx), TransactionReordering{
            inputs: vec![1, 0],
            outputs: vec![1, 0],
        });
        assert_eq!(tx, get_bip69_ex1());
    }

    #[test]
    fn test_reorder_tx_2() {
        let mut tx = get_bip69_ex2();
        assert_eq!(reorder_tx(&mut tx), TransactionReordering{
            inputs: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            outputs: vec![0, 1],
        });
        assert_eq!(tx, get_bip69_ex2());

        TransactionReordering{
            inputs: vec![16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
            outputs: vec![1, 0],
        }.apply(&mut tx);

        assert_eq!(reorder_tx(&mut tx), TransactionReordering{
            inputs: vec![16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
            outputs: vec![1, 0],
        });
        assert_eq!(tx, get_bip69_ex2());
    }
}
