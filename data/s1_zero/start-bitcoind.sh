#!/bin/bash -e

mkdir -p data/bitcoind

bitcoind \
    -blocknotify="echo New block %s" \
    -datadir=data/bitcoind \
    -txindex \
    -debug=1 \
    -printtoconsole \
    -server \
    -regtest \
    -rpcport=18443 \
    -rpcuser=user \
    -rpcpassword=password \
    -zmqpubrawblock=tcp://127.0.0.1:18501 \
    -zmqpubrawtx=tcp://127.0.0.1:18501 \
    $@