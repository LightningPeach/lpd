#!/usr/bin/env bash

electrs \
    -vvv \
    --network=regtest \
    --jsonrpc-import \
    --daemon-dir=data/bitcoind \
    --db-dir=data/electrs-db \
    --cookie=user:password \
    --daemon-rpc-addr=127.0.0.1:18443
