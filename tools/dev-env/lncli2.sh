#!/bin/bash

lncli \
    --macaroonpath data/lnd2/admin.macaroon \
    --rpcserver localhost:11009 \
    --tlscertpath data/lnd2/tls.cert \
    $@