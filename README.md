## Lightning Peach Daemon

The Lightning Peach Daemon (`lpd`) - is a partial implementation of a
[Lightning Network](https://lightning.network) node in Rust.

Work is still in early stages. Currently near 20% towards usable production ready system.

The goal is to provide a full-featured implementation of Lightning with enhanced security (due to a RUST usage)
which potentially can be run on many platforms including WASM.

As a reference implementation [lnd] (https://github.com/lightningnetwork/lnd) was used. 

## Lightning Network Specification Compliance
`lpd` _partially_ implements the [Lightning Network specification
(BOLTs)](https://github.com/lightningnetwork/lightning-rfc). BOLT stands for:
Basic of Lightning Technologies.

- [partial]         BOLT 1: Base Protocol
- [partial]         BOLT 2: Peer Protocol for Channel Management
- [full]            BOLT 3: Bitcoin Transaction and Script Formats
- [full]            BOLT 4: Onion Routing Protocol
- [not implemented] BOLT 5: Recommendations for On-chain Transaction Handling
- [partial]         BOLT 7: P2P Node and Channel Discovery
- [full]            BOLT 8: Encrypted and Authenticated Transport
- [partial]         BOLT 9: Assigned Feature Flags
- [not implemented] BOLT 10: DNS Bootstrap and Assisted Node Location
- [not implemented] BOLT 11: Invoice Protocol for Lightning Payments

Currently development is done in a few separate branches that will be merged in master when they are ready.
- routing - BOLT-7
- state_machine - BOLT-2, BOLT-3

We firmly believe in the share early, share often approach. The basic premise of the approach is to announce your plans 
before you start work, and once you have started working share your work when any progress is made.
Do not wait for a one large pull request.

Communication is done using github issues. If you have an idea, issue with code or want to contribute create
an issue on github.