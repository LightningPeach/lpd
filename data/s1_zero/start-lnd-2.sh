#!/bin/bash

mkdir -p data/lnd2

lnd \
    --lnddir=data/lnd2 \
    --configfile=data/lnd2/lnd.conf \
    --tlscertpath=data/lnd2/tls.cert \
    --tlskeypath=data/lnd2/tls.key \
    --adminmacaroonpath=data/lnd2/admin.macaroon \
    --readonlymacaroonpath=data/lnd2/readonly.macaroon \
    --invoicemacaroonpath=data/lnd2/invoice.macaroon \
    --rpclisten=localhost:11009 \
    --restlisten=localhost:11010 \
    --listen=:11011 \
    --externalip=localhost:11011 \
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
