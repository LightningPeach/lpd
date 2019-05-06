#!/usr/bin/env bash

CURR_DIR=$(dirname $(realpath $0))

TMP_DIR=$(mktemp -d -t dev-env-XXXXXXXXXX)
echo "Created env dir:" $TMP_DIR

cp $CURR_DIR/*.sh $TMP_DIR

tilix -a app-new-window \
    -t "Dev ENV" \
    -w $TMP_DIR \
    -x 'zsh -c "DE_CREATE_NEW_ENV=1 EVAL_AT_START=./_01-start-bitcoind-lnd3.sh zsh"'