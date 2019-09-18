use super::super::{Home, cleanup};
use super::{BitcoinConfig, BitcoinInstance};
use crate::home::create_file_for_redirect;

use std::process::{Command, Child};
use std::{io, fs};
use bitcoin_rpc_client::{Client, Error};
use std::fs::File;
use std::path::PathBuf;

pub struct Bitcoind {
    home: Home,
    // Listen for connections on <port> (default: 8333 or testnet: 18333)
    port: u16,
}

pub struct BitcoindRunning {
    config: Bitcoind,
    instance: Child,
    stdout: File,
    stderr: File
}

impl AsMut<Bitcoind> for BitcoindRunning {
    fn as_mut(&mut self) -> &mut Bitcoind {
        &mut self.config
    }
}

impl AsRef<Bitcoind> for BitcoindRunning {
    fn as_ref(&self) -> &Bitcoind {
        &self.config
    }
}

impl Drop for BitcoindRunning {
    fn drop(&mut self) {
        self.instance.kill()
            .or_else(|e| match e.kind() {
                io::ErrorKind::InvalidInput => Ok(()),
                _ => Err(e),
            })
            .unwrap()
    }
}

impl BitcoinConfig for Bitcoind {
    type Instance = BitcoindRunning;

    fn new(name: &str) -> Result<Self, io::Error> {
        Ok(Bitcoind {
            home: Home::new(name, false, true)
                .or_else(|e| if e.kind() == io::ErrorKind::AlreadyExists {
                    // TODO(mkl): this should be fixed to not delete all bitcoind processes
                    cleanup("bitcoind");
                    Home::new(name, true, true)
                } else {
                    Err(e)
                })?,
            port: 18333,
        })
    }

    fn run(self) -> Result<Self::Instance, io::Error> {
        self.run_internal()
    }

    fn lnd_params(&self) -> Vec<String> {
        vec![
            "--bitcoin.active".to_owned(),
            "--bitcoin.regtest".to_owned(),
            "--bitcoin.node=bitcoind".to_owned(),
            "--bitcoind.rpcuser=devuser".to_owned(),
            "--bitcoind.rpcpass=devpass".to_owned(),
            "--bitcoind.zmqpubrawblock=tcp://127.0.0.1:18501".to_owned(),
            "--bitcoind.zmqpubrawtx=tcp://127.0.0.1:18502".to_owned(),
            "--bitcoind.rpchost=localhost:18443".to_owned(),
            format!("--bitcoind.dir={}", self.home.path().to_str().unwrap())
        ]
    }
}

impl Bitcoind {
    pub fn set_peer_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn stdout_path(&self) -> PathBuf {
        self.home.ext_path("bitcoind.stdout")
    }

    pub fn stderr_path(&self) -> PathBuf {
        self.home.ext_path("bitcoind.stderr")
    }

    fn run_internal(self) -> Result<BitcoindRunning, io::Error> {
        let data_dir_path= self.home.ext_path("data");
        fs::create_dir(&data_dir_path).or_else(|e|
            if e.kind() == io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(e)
            }
        ).map_err(|err| {
            println!("cannot create data dir for bitcoind: {:?} , error: {:?}", &data_dir_path, err);
            err
        })?;

        let args = vec![
            format!("-datadir={}", data_dir_path.to_str().unwrap()),
            format!("-port={}", self.port),
        ];

        let (stdout, stdout_file) = create_file_for_redirect(self.stdout_path()).map_err(|err|{
            println!("cannot create file {:?} for bitcoind stdout: {:?}", &self.stdout_path(), err);
            err
        })?;

        let (stderr, stderr_file) = create_file_for_redirect(self.stderr_path()).map_err(|err| {
            println!("cannot create file {:?} for bitcoind stderr: {:?}", &self.stderr_path(), err);
            err
        })?;

        Command::new("bitcoind")
            .args(&[
                "-regtest", "-server", "-txindex", "-rpcuser=devuser", "-rpcpassword=devpass",
                "-deprecatedrpc=generate",
                "-rpcport=18443",
                "-zmqpubrawblock=tcp://127.0.0.1:18501", "-zmqpubrawtx=tcp://127.0.0.1:18502",
            ])
            .args(args)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .map(|instance| BitcoindRunning {
                config: self,
                instance: instance,
                stdout: stdout_file,
                stderr: stderr_file
            })
    }
}

impl BitcoinInstance for BitcoindRunning {
    fn rpc_client(&self) -> Result<Client, Error> {
        use bitcoin_rpc_client::Auth::UserPass;

        Client::new("http://localhost:18443".to_owned(), UserPass("devuser".to_owned(), "devpass".to_owned()))
    }
}
