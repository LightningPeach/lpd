#!/usr/bin/env bash

# we need to stop eny existing bitcoind so they do not interfere with us
# we cannot kill with -9 because it screw up level db and
# at restart some blocks will be missing
pkill bitcoind

# Kill any existing lnd (and electrs) because they may interfere with our env
pkill -9 'electrs|lnd'

tilix -a app-new-session \
    -t "bitcoind" \
    -x 'zsh -c "EVAL_AT_START=./start-bitcoind.sh zsh"'

# Wait some time until bitcoind fully start
sleep 3

tilix \
    -a app-new-session \
    -t "electrs" \
    -x 'zsh -c "EVAL_AT_START=./start-electrs.sh zsh"'

sleep 1

# If we create new env then mine some blocks to enable SEGWIT
if [ -n "$DE_CREATE_NEW_ENV" ]; then
    ./bitcoin-cli.sh generate 500
    sleep 2
fi


tilix \
    -a app-new-session \
    -t "lnd1" \
    -x 'zsh -c "EVAL_AT_START=./start-lnd-1.sh zsh"'
sleep 4

tilix \
    -a app-new-session \
    -t "lnd2" \
    -x 'zsh -c "EVAL_AT_START=./start-lnd-2.sh zsh"'
sleep 4

# Generate block, so lnds updates its status
./bitcoin-cli.sh generate 1
sleep 1

if [ -n "$DE_CREATE_NEW_ENV" ]; then
    LND1_WALLET_ADDR=$(./lncli1.sh newaddress np2wkh | jq -r ".address")
    ./bitcoin-cli.sh generatetoaddress 10 $LND1_WALLET_ADDR
    sleep 1

    # Generate 100 blocks to enable coinbase outputs
    ./bitcoin-cli.sh generate 100
    sleep 5

    # Connect first lnd to second
    LND1_PEER_ADDR=$(./lncli1.sh getinfo | jq -r '.uris[0]')
    ./lncli2.sh connect $LND1_PEER_ADDR

    # Open channel
    LND2_IDENTITY_PUBKEY=$(./lncli2.sh getinfo | jq -r '.identity_pubkey')
    ./lncli1.sh openchannel $LND2_IDENTITY_PUBKEY 100000
    ./bitcoin-cli.sh generate 3
    sleep 1

    # Pay from first node to second
    PAYMENT_REQUEST=$(./lncli2.sh addinvoice 1000 | jq -r '.pay_req')
    echo $PAYMENT_REQUEST
    ./lncli1.sh payinvoice -f $PAYMENT_REQUEST
fi
