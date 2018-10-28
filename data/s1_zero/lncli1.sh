#!/bin/bash

lncli \
    --macaroonpath data/lnd1/admin.macaroon \
    --rpcserver localhost:10009 \
    --tlscertpath data/lnd1/tls.cert \
    $@