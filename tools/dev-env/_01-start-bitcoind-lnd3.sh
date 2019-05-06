#!/usr/bin/env bash

# we need to stop eny existing bitcoind so they do not interfere with us
# we cannot kill with -9 because it screw up level db and
# at restart some blocks will be missing
pkill bitcoind

# Sometimes thay hang-up so we need to kill with -9
pkill -9 "lnd|electrs"

tilix \
    -a app-new-session \
    -t "bitcoind" \
    -x 'zsh -c "EVAL_AT_START=./start-bitcoind.sh zsh"'

# Wait some time until bitcoind fully start
sleep 2

# If we create new env then mine some blocks to enable SEGWIT
if [ -n "$DE_CREATE_NEW_ENV" ]; then
    ./bitcoin-cli.sh generate 500
fi

tilix \
    -a app-new-session \
    -t "electrs" \
    -x 'zsh -c "EVAL_AT_START=./start-electrs.sh zsh"'

sleep 1

tilix \
    -a app-new-session \
    -t "lnd3" \
    -x 'zsh -c "EVAL_AT_START=./start-lnd-3.sh zsh"'

sleep 5

if [ -n "$DE_CREATE_NEW_ENV" ]; then
    LND3_WALLET_ADDR=$(./lncli3.sh newaddress np2wkh | jq -r ".address")
    ./bitcoin-cli.sh generatetoaddress 10 $LND3_WALLET_ADDR
    sleep 2

    # Generate 100 blocks to enable coinbase outputs
    ./bitcoin-cli.sh generate 100
    sleep 5
fi

# Generate block, so lnd updates its status
./bitcoin-cli.sh generate 1