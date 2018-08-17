#!/bin/bash -e

bitcoin-cli \
    -datadir=data/bitcoind \
    -regtest \
    -rpcport=18443 \
    -rpcuser=user \
    -rpcpassword=password \
    $@