[Article on Medium](https://medium.com/@bramcohen/how-wallets-can-handle-transaction-fees-ff5d020d14fb)
[About mempool on Medium](https://blog.kaiko.com/an-in-depth-guide-into-how-the-mempool-works-c758b781c608)

Market fee density represents balance between supply and demand. 
It determines the price in units `satoshi / vbyte`.
`vbyte` is the size of transaction in bytes.

Market fee density could be calculated using last few blocks and mempool content.

There is a [probabilistic model](https://github.com/bitcoinfees/feesim) for such calculations.
The model's input parameters are 
the desired probability of successfull confirmation and 
the number of blocks during which the confirmation happens.
The model's output is the price in units `satoshi / vbyte`.

Two assumption holds for fee prediction: the miners takes more fee first 
and fee density is the same as before (on previous block).

[Implementation at btcd.](https://github.com/btcsuite/btcd/blob/master/mempool/estimatefee.go)
It observes blocks and transactions and calculates median fee in the predefined amount of blocks.
