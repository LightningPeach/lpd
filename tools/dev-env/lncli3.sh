#!/bin/bash

lncli \
    --macaroonpath data/lnd3/admin.macaroon \
    --rpcserver localhost:12009 \
    --tlscertpath data/lnd3/tls.cert \
    $@