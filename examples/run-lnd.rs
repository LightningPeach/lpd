#!/usr/bin/env run-cargo-script

// required: gencerts, btcd, lnd

use std::fs;
use std::process::Output;
use std::process::Command;
use std::io;
use std::string::String;

fn main() {
    // TODO: workdir macro

    fs::create_dir_all("/tmp/healthcheck/btcd")
        .or_else(|err| if err.kind() == io::ErrorKind::NotFound { Ok(()) } else { Err(err) })
        .unwrap();

    Command::new("gencerts")
        .current_dir("/tmp/healthcheck/btcd").output().unwrap();
    let btcd = Command::new("btcd")
        .args(&["--simnet", "--txindex", "--rpcuser=devuser", "--rpcpass=devpass",
            "--datadir=/tmp/healthcheck/btcd/data", "--logdir=/tmp/healthcheck/btcd/logs",
            "--rpccert=/tmp/healthcheck/btcd/rpc.cert", "--rpckey=/tmp/healthcheck/btcd/rpc.key",
            "--miningaddr=sb1qvc0mwkl35rl60memjwglxjnz0qsfxhaqq3nx4x"
        ]).spawn().unwrap();
    let lnd = Command::new("lnd")
        .args(&["--bitcoin.active", "--bitcoin.simnet", "--noencryptwallet", "--no-macaroons",
            "--btcd.rpcuser=devuser", "--btcd.rpcpass=devpass",
            "--datadir=/tmp/healthcheck/lnd-node1/data", "--logdir=/tmp/healthcheck/lnd-node1/logs",
            "--tlscertpath=/tmp/healthcheck/btcd/rpc.cert",
            "--tlskeypath=/tmp/healthcheck/btcd/rpc.key",
            "--btcd.rpccert=/tmp/healthcheck/btcd/rpc.cert", "--listen=localhost:10011",
            "--rpclisten=localhost:10009", "--restlisten=localhost:8000"
        ]).spawn().unwrap();
    let info = Command::new("lncli")
        .current_dir("/tmp/healthcheck")
        .args(&["--rpcserver", "localhost:10009", "--tlscertpath", "btcd/rpc.cert", "getinfo"])
        .output().check();
    println!("{:?}", info);

    let _ = lnd.wait_with_output().check();
    let _ = btcd.wait_with_output().check();
}

trait CheckOutput {
    fn check(self) -> String;
}

impl CheckOutput for Result<Output, io::Error> {
    fn check(self) -> String {
        let output = self.unwrap();
        if output.status.success() {
            String::from_utf8(output.stdout).unwrap()
        } else {
            panic!("{:?}", String::from_utf8(output.stderr));
        }
    }
}
