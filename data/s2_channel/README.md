# Scripts for helping developing lpd

This directory contains shell script for simplification of developing code
They use slightly modified version of lnd with enabled message dumping. Basically it writes in file all messages received or sent.
To debug Rust programs CLion is needed, Intellij with Rust plugin will not support debugging.

# Assumed environment
- Linux
- Clion (not necessary)
- Rust > 1.31
- zsh It is possible to use other shells. 
- tilix https://gnunn1.github.io/tilix-web/ - modern terminal. 

# Installation
1. Install lnd TODO(mkl): link to it. lnd is considered as reference implementation. lpd should be fully compatible with it.
   https://github.com/lightningnetwork/lnd/blob/master/docs/INSTALL.md
2. Install tilix (optional)
   https://gnunn1.github.io/tilix-web/
3. Install lmsg (optional): ...


# Common tasks

Note: some of this scripts kill already running bitcoind, lnd. 
It often occurs the folowing tasks:
1. Create new regtest bitcoin blockchain. Mine some blocks. Launch lnd
2. Create new regtest bitcoin blockchain. Mine some blocks. Launch few lnds. Create channel between them
3. Create new regtest bitcoin blockchain. Mine some blocks. Launch electrum server. Launch lnd. Money some money to it. 

# List of scripts

## _start-tilix-debug.sh 
is internal script. In creates 3 tilix sessions:
 - one with terminal
 - one with launched bitcoind
 - one with launched lnd (lnd3) 

Then mines one block so lnd updates its status.

##_start-tilix-new-env.sh  
Creates a new environment. Note: it kills all bitcoind, lnd at start.
1. Create new session. Launch bitcoind in it.
2. Mine 500 blocks. So Segwit get activated
3. Launch lnd1, wait some time
4. Launch lnd2, wait some time
5. Generate 1 block so lnd updates its statuses
6. Mine some blocks to address of the first lnd
7. Mine 100 blocks to enable spending of coinbase outputs
8. Connect 2nd lnd to first
9. Open channel from first channel to second 
10. Generate 3 blocks
11. Make payment from 1st lnd to 2nd

##bitcoin-cli.sh 
is wrapper around bitcoin-cli with connection parameters

## lncli1.sh 
is a wrapper around lncli so it connects to lnd1

## lncli2.sh 
is a wrapper around lncli so it connects to lnd2

## lncli3.sh 
is a wrapper around lncli so it connects to lnd3

## new_temp_env.sh 
launch new environment in temporary dir.
1. Creates temp dir
2. Copies scripts into it
3. Launches new tilix session from this dir using _start-tilix-new-env.sh

## open_channel.sh 
opens channel with (first) peer of lnd1. 

## start-bitcoind.sh 
starts bitcoind

## start-electrs.sh 
starts electrs (TODO: link to explanation what is it)

## start-lnd-1.sh 
starts lnd1

## start-lnd-1-redirect.sh 
start lnd1 with redirecting stout and stderr in a file

## start-lnd-2.sh 
starts lnd2

## start-lnd-3.sh 
starts lnd3

## start-tilix-debug.sh 
starts tilix debug session ( TODO(mkl): explain)


## start-tilix-new-env.sh 
starts tilix new env session ( TODO(mkl): explain )

 
Tilix commands:

# Create new session (basically tab)
(maybe add example)
$ tilix -a app-new-session 

To launch shell and then launch command in it 
$ tilix -x 'zsh -c "EVAL_AT_START=./start-lnd-3.sh zsh"'

To work it uses 
add this to '~/.zshrc'
```
if [ -n "$EVAL_AT_START" ];
then
eval "$EVAL_AT_START"
fi
```

Developing node requires some tools:
1. `bitcoind` - original Bitcoin implementation. There are others like `btcd` and `parity`.
2. `lnd` - one of existing Lightning implementations. We consider it as a reference. `lpd` should be fully compatible with `lnd`
3. `electrs` - Electrum server in Rust. https://github.com/romanz/electrs . Currently lpd uses wallet that uses electrum
4. `tilix` - modern terminal. It is used to open multiple programs in different tabs.
5. `lmsg` - (optional) Tool for creating, decoding/encoding Lightning messages


This directory contains a few scripts. You launch them and obtain regtest network with some preconfiguration, which you can use for work.
 
TODO(mkl):
1. Explain what components are used
2. How to install
 - bitcoind 
 - lnd
 - electrs
 - tilix
3. How parts fit together
4. Move all constants into variables
5. What tasks should be done to configure lnd ?


Explain main tools in this package:
1. server rpc/server  - contains implementation of node
2. client rpc/client  - contains client for this node
3. wire-compatibility - tool for encoding/decoding Lightning messages. It is used to check compatibility 
4. dump-reader - tool for reading message dump diagrams. It gives statistics or diagram 
