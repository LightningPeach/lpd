#!/usr/bin/env bash

tilix -a app-new-session -t "console"

EVAL_AT_START="./start-bitcoind.sh"
tilix -a app-new-session -t "bitcoind" -x 'zsh -c "EVAL_AT_START=./start-bitcoind.sh zsh"'

# Wait some time until bitcoind fully start
sleep 1

tilix -a app-new-session -t "lnd3" -x 'zsh -c "EVAL_AT_START=./start-lnd-3.sh zsh"'

sleep 2

# Generate block, so lnd updates its status
./bitcoin-cli.sh generate 1