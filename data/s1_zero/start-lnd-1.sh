#!/bin/bash

mkdir -p data/lnd1

lnd \
    --lnddir=data/lnd1 \
    --configfile=data/lnd1/lnd.conf \
    --tlscertpath=data/lnd1/tls.cert \
    --tlskeypath=data/lnd1/tls.key \
    --adminmacaroonpath=data/lnd1/admin.macaroon \
    --readonlymacaroonpath=data/lnd1/readonly.macaroon \
    --invoicemacaroonpath=data/lnd1/invoice.macaroon \
    --rpclisten=localhost:10009 \
    --restlisten=localhost:10010 \
    --listen=:10011 \
    --externalip=localhost:10011 \
    --debuglevel=info \
    --nobootstrap \
    --noencryptwallet \
    --bitcoin.active \
    --bitcoin.node=bitcoind \
    --bitcoin.regtest \
    --bitcoind.dir=data/bitcoind \
    --bitcoind.rpchost=localhost:18443 \
    --bitcoind.rpcuser=user \
    --bitcoind.rpcpass=password \
    --bitcoind.zmqpath=tcp://127.0.0.1:18501
