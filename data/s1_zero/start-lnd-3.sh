#!/bin/bash

mkdir -p data/lnd3

lnd \
    --lnddir=data/lnd3 \
    --configfile=data/lnd3/lnd.conf \
    --tlscertpath=data/lnd3/tls.cert \
    --tlskeypath=data/lnd3/tls.key \
    --adminmacaroonpath=data/lnd3/admin.macaroon \
    --readonlymacaroonpath=data/lnd3/readonly.macaroon \
    --invoicemacaroonpath=data/lnd3/invoice.macaroon \
    --rpclisten=localhost:12009 \
    --restlisten=localhost:12010 \
    --listen=:12011 \
    --externalip=localhost:12011 \
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
