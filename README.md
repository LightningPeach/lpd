## Lightning Peach Daemon

The Lightning Peach Daemon (`lpd`) - is a partial implementation of a
[Lightning Network](https://lightning.network) node in Rust.

Work is still in early stages. Currently near 20% towards usable production ready system.

The goal is to provide a full-featured implementation of Lightning with enhanced security (due to a Rust usage)
which potentially can be run on many platforms including WASM.

As a reference implementation [lnd] (https://github.com/lightningnetwork/lnd) was used. 

## Lightning Network Specification Compliance
`lpd` _partially_ implements the [Lightning Network specification
(BOLTs)](https://github.com/lightningnetwork/lightning-rfc). BOLT stands for:
Basic of Lightning Technologies.

- [partial]         BOLT 1: Base Protocol
- [partial]         BOLT 2: Peer Protocol for Channel Management
- [partial]         BOLT 3: Bitcoin Transaction and Script Formats
- [full]            BOLT 4: Onion Routing Protocol
- [not implemented] BOLT 5: Recommendations for On-chain Transaction Handling
- [partial]         BOLT 7: P2P Node and Channel Discovery
- [full]            BOLT 8: Encrypted and Authenticated Transport
- [partial]         BOLT 9: Assigned Feature Flags
- [not implemented] BOLT 10: DNS Bootstrap and Assisted Node Location
- [not implemented] BOLT 11: Invoice Protocol for Lightning Payments

See `rpc/interface/*.proto` for rpc interface.

Bitcoin daemon and Electrum Server should be running before run lpd server.

Install `electrs` with:

    git clone https://github.com/romanz/electrs &&
    cd electrs &&
    git checkout bb62df8793948b88cb2bc61580ca727cbbae9d31 &&
    cargo install --debug --path .

Run `bitcoind`:

`bitcoind -txindex -regtest -rpcport=18443 -rpcuser=devuser -rpcpassword=devpass -zmqpubrawblock=tcp://127.0.0.1:18501 -zmqpubrawtx=tcp://127.0.0.1:18502`

Run `electrs`:

`electrs --network=regtest --jsonrpc-import --cookie=devuser:devpass --daemon-rpc-addr=127.0.0.1:18443`

Generate some blocks:

`bitcoin-cli -rpcport=18443 -rpcuser=devuser -rpcpassword=devpass generate 1`

The command for running the rpc server is:

`cargo run --package server`

or 

`cargo run --package server --release`

for release configuration. Cli parameters are following:

- `--rpclisten=0.0.0.0:1234` ip and port to listen RPC on. Default is `127.0.0.1:10009`.
- `--listen=0.0.0.0:1234` ip and port to listen peers on. Default is `127.0.0.1:9735`.
- `--db-path=path/to/database` path to database directory. Default is `target/db`.

CLI parameters are passed through `--`, for example

`cargo run --package server -- --db-path=tmp/somedb --listen=1.2.3.4:1234`

We firmly believe in the share early, share often approach. The basic premise of the approach is to announce your plans 
before you start work, and once you have started working share your work when any progress is made.
Do not wait for a one large pull request.

Communication is done using github issues. If you have an idea, issue with code or want to contribute create
an issue on github.