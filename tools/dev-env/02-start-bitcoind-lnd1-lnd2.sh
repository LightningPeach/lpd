#!/usr/bin/env bash

CURR_DIR=$(dirname $(realpath $0))

tilix -a app-new-window \
    -t "Dev ENV" \
    -w $CURR_DIR \
    -x 'zsh -c "EVAL_AT_START=./_02-start-bitcoind-lnd1-lnd2.sh zsh"'