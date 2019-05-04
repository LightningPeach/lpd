#!/bin/bash -e

CHANNEL_SIZE=100000
BASE_PATH=$(dirname $0)
PRIV_KEY=$($BASE_PATH/lncli1.sh listpeers | jq -e -r '.peers[0].pub_key')
echo "Node pubkey:" $PRIV_KEY
$BASE_PATH/lncli1.sh openchannel $PRIV_KEY $CHANNEL_SIZE
$BASE_PATH/bitcoin-cli.sh generate 1

