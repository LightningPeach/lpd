#!/usr/bin/env bash

echo "creating new temp env"

# Plan
# 1. Create some temporary dir
# 2. Copy all needed files into it
# 3. Start tilix with this dir (similar to start-tilix-debug.sh)
# 4. Add opening new channel to it

TMP_DIR=$(mktemp -d -t lnd-env-XXXXXXXXXX)
echo "Created env dir:" $TMP_DIR

cp start-bitcoind.sh \
   bitcoin-cli.sh \
   start-lnd-1.sh \
   start-lnd-2.sh \
   lncli1.sh \
   lncli2.sh \
   _start-tilix-new-env.sh \
   $TMP_DIR

cd $TMP_DIR

tilix -a app-new-window -t "Debug ENV" -w $TMP_DIR -x 'zsh -c "EVAL_AT_START=./_start-tilix-new-env.sh zsh"'

