# LND to LPD interface differences

## Remote Procedure Call

### New types `Satoshi` and `MilliSatoshi`

Amount of satoshi is typed. There are two new types that should be used instead of `int`

## Command Line Interface

### TLS Acceptor

The server optionally takes p12 to establish secure connection for RPC. 
There are two parameters `--pkcs12=path/to/p12` and `--pkcs12-password=qwerty123`.

See README.md for more CLI parameters.
