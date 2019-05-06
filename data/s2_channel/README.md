# Scripts for helping developing lpd

This directory contains a few scripts. You launch them and obtain regtest network with some preconfiguration, which you can use for work.

They use slightly modified version of lnd with enabled message dumping. Basically it writes in file all messages received or sent.

To debug Rust programs CLion is needed, Intellij with Rust plugin will not support debugging.

# Assumed environment
- Linux (or Mac)
- Clion (not necessary)
- Rust > 1.31
- zsh (It is possible to use other shells but it requires changing `zsh` to `your_shell` everywhere) 
- tilix https://gnunn1.github.io/tilix-web/ - modern terminal. 

# Installation
1. Install lnd. lnd is considered as reference implementation. lpd should be fully compatible with it.
   https://github.com/lightningnetwork/lnd/blob/master/docs/INSTALL.md
2. Install tilix
   https://gnunn1.github.io/tilix-web/
3. Install lmsg (optional): ...
4. add this to '~/.zshrc'
   ```
   if [ -n "$EVAL_AT_START" ];
   then
   eval "$EVAL_AT_START"
   fi
   ```


# Common tasks

Note: this scripts kill already running bitcoind, lnd, electrs.
 
1. Create new regtest bitcoin blockchain. Start electrs. Mine some blocks. Launch lnd. **Solution: `01-create-bitcoind-lnd3.sh`**

2. Start regtest bitcoin blockchain. Start electrs. Launch lnd. **Solution: go to dir with env and launch `01-start-bitcoind-lnd3.sh`**

3. Create new regtest bitcoin blockchain. Start electrs. Mine some blocks. Launch two lnds. Create channel between them and make payment. **Solution: `02-create-bitcoind-lnd1-lnd2.sh`**

4. Start regtest bitcoin blockchain. Start electrs. Launch lnds. **Solution: go to dir with env and launch `02-start-bitcoind-lnd1-lnd2.sh`**

# List of scripts
All environmen variables used here have prefix `DE_` (from Dev Environment).
During launch each program is launched in its own terminal session (similar to as it would be done by hand)

## `01-create-bitcoind-lnd3.sh` 
creates new env(bitcoind, electrs, lnd3) in temporary dir
Internally:
    1. Create new temporaty dir
    2. Copy *.sh files into it
    3. Start new terminal with `_01-start-bitcoind-lnd3.sh` and DE_CREATE_NEW_ENV=1

## `01-start-bitcoind-lnd3.sh` 
starts env (bitcoind, lnd, electrs) in current dir
Internally:
 1. Start new terminal with `_01-start-bitcoind-lnd3.sh`. NOTE: `DE_CREATE_NEW_ENV=1` is NOT set

## `_01-start-bitcoind-lnd3.sh` - used inernally
It is internal script and should be not launched directly. It launches programs in terminals. If DE_CREATE_NEW_ENV is set mines initial blocks, ...

## `02-create-bitcoind-lnd1-lnd2.sh`
create new env(bitcoind, lnd1, lnd2, electrs) in temporary dir. Opens channel from lnd1 to lnd2. Make one payment from lnd1 to lnd2
Internally:
1. Create new temporary dir
2. Copy *.sh files into it
3. Start new terminal with `_02-start-bitcoind-lnd1-lnd2.sh` and DE_CREATE_NEW_ENV=1

## `02-start-bitcoind-lnd1-lnd2.sh` 
start env(bitcoind, lnd1, lnd2, electrs) is current dir.

## `_02-start-bitcoind-lnd1-lnd2.sh` - used internally 
It is internal script and should be not launched directly. It launches programs in terminals. If  `DE_CREATE_NEW_ENV=1` do some initial work: mine blocks, connect lnd1 to lnd2, ...

## `bitcoin-cli.sh` 
is wrapper around bitcoin-cli with connection parameters

## `lncli1.sh` 
is a wrapper around lncli so it connects to lnd1

## `lncli2.sh` 
is a wrapper around lncli so it connects to lnd2

## `lncli3.sh` 
is a wrapper around lncli so it connects to lnd3

## `open_channel.sh` 
opens channel with (first) peer of lnd3. 

## `start-bitcoind.sh` 
starts bitcoind

## `start-electrs.sh` 
starts electrs

## `start-lnd-1.sh`
starts lnd1

## `start-lnd-1-redirect.sh` 
start lnd1 with redirecting stout and stderr in a file

## `start-lnd-2.sh` 
starts lnd2

## `start-lnd-3.sh` 
starts lnd3

 
# Why Tilix is used:

## Create new session (basically tab)
$ tilix -a app-new-session 

## To launch shell and then launch command in it 
$ tilix -x 'zsh -c "EVAL_AT_START=./start-lnd-3.sh zsh"'

To work it uses 
add this to '~/.zshrc'
```
if [ -n "$EVAL_AT_START" ];
then
eval "$EVAL_AT_START"
fi
```

# Developing node requires some tools

1. `bitcoind` - original Bitcoin implementation. There are others like `btcd` and `parity`.
2. `lnd` - one of existing Lightning implementations. We consider it as a reference. `lpd` should be fully compatible with `lnd`
3. `electrs` - Electrum server in Rust. https://github.com/romanz/electrs . Currently lpd uses wallet that uses electrum
4. `tilix` - modern terminal. It is used to open multiple programs in different tabs.
5. `lmsg` - (optional) Tool for creating, decoding/encoding Lightning messages



# Main executables in lpd:
1. server rpc/server  - contains implementation of node
2. client rpc/client  - contains client for this node
3. wire-compatibility - tool for encoding/decoding Lightning messages. It is used to check compatibility 
4. dump-reader - tool for reading message dump diagrams. It gives statistics or diagram 
