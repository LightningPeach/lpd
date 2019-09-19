use super::super::{Home, cleanup};
use super::{BitcoinConfig, BitcoinInstance};
use crate::home::{create_file_for_redirect, write_to_file};
use crate::error::Error;
use crate::{new_io_error, new_bitcoin_rpc_error, new_error};

use std::process::{Command, Child};
use std::{io, fs};
use bitcoin_rpc_client::{Client, Error as BitcoinRpcError};
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct BitcoindConfig {
    pub home: Home,

    // Should the process be killed in the end
    // It might be useful if you want to play with the bitcoind process after tests finish
    pub kill_in_the_end: bool,

    // Listen for connections on <port> (default: 8333 or testnet: 18333)
    pub port: u16,
}

pub struct BitcoindProcess {
    pub config: BitcoindConfig,
    pub instance: Child,

    stdout: File,
    stderr: File
}

impl AsMut<BitcoindConfig> for BitcoindProcess {
    fn as_mut(&mut self) -> &mut BitcoindConfig {
        &mut self.config
    }
}

impl AsRef<BitcoindConfig> for BitcoindProcess {
    fn as_ref(&self) -> &BitcoindConfig {
        &self.config
    }
}

impl Drop for BitcoindProcess {
    fn drop(&mut self) {
        if self.config.kill_in_the_end {
            self.instance.kill()
                .or_else(|e| match e.kind() {
                    io::ErrorKind::InvalidInput => Ok(()),
                    _ => Err(e),
                })
                .unwrap()
        }
    }
}

impl BitcoinConfig for BitcoindConfig {
    type Instance = BitcoindProcess;

    fn new(name: &str) -> Result<Self, Error> {
        Ok(BitcoindConfig {
            // TODO(mkl): rewrite this part. What is the problem with force delete?
            home: Home::new(name, false, false)?,
//                .or_else(|e| if e.kind() == io::ErrorKind::AlreadyExists {
//                    // TODO(mkl): this should be fixed to not delete all bitcoind processes
//                    cleanup("bitcoind");
//                    Home::new(name, true, true)
//                } else {
//                    Err(e)
//                })?,
            port: 18333,
            kill_in_the_end: false,
        })
    }

    fn run(self) -> Result<Self::Instance, Error> {
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

impl BitcoindConfig {
    pub fn set_peer_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn stdout_path(&self) -> PathBuf {
        self.home.ext_path("bitcoind.stdout")
    }

    pub fn stderr_path(&self) -> PathBuf {
        self.home.ext_path("bitcoind.stderr")
    }

    pub fn pid_path(&self) -> PathBuf {
        self.home.ext_path("bitcoind.pid")
    }

    fn run_internal(self) -> Result<BitcoindProcess, Error> {
        let data_dir_path= self.home.ext_path("data");
        fs::create_dir(&data_dir_path).or_else(|e|
            if e.kind() == io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(e)
            }
        ).map_err(|err| {
            new_io_error!(
                err,
                "cannot create data dir for bitcoind",
                data_dir_path.to_string_lossy().into_owned()
            )
        })?;

        let args = vec![
            format!("-datadir={}", data_dir_path.to_str().unwrap()),
            format!("-port={}", self.port),
        ];

        let (stdout, stdout_file) = create_file_for_redirect(self.stdout_path()).map_err(|err|{
            new_io_error!(
                err,
                "cannot create file for bitcoind stdout",
                self.stdout_path().to_string_lossy().into_owned()
            )
        })?;

        let (stderr, stderr_file) = create_file_for_redirect(self.stderr_path()).map_err(|err| {
            new_io_error!(
                err,
                "cannot create file for bitcoind stderr",
                self.stderr_path().to_string_lossy().into_owned()
            )
        })?;

        let pid_path = self.pid_path();
        // TODO(mkl): refactor arguments in new method
        let bitcoind_process = Command::new("bitcoind")
            // TODO(mkl): move all this configuration into variables
            // TODO(mkl): check ports if they are not used
            // TODO(mkl): handle errors
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
            .map(|instance| BitcoindProcess {
                config: self,
                instance: instance,
                stdout: stdout_file,
                stderr: stderr_file
            })
            .map_err(|err| {
                new_io_error!(err, "cannot launch bitcoind")
            })?;

        let pid_str = format!("{}", bitcoind_process.instance.id());
        write_to_file(&pid_path, &pid_str)
            .map_err(|err| {
                new_error!(err, "cannot write bitcoind pid file")
            })?;
        Ok(bitcoind_process)
    }
}

impl BitcoinInstance for BitcoindProcess {
    fn rpc_client(&self) -> Result<Client, Error> {
        use bitcoin_rpc_client::Auth::UserPass;

        // TODO(mkl): refactor config parameters
        Client::new("http://localhost:18443".to_owned(), UserPass("devuser".to_owned(), "devpass".to_owned()))
            .map_err(|err| {
                // TODO(mkl): Maybe include exact information about connection like user, password, ...
                new_bitcoin_rpc_error!(err, "cannot connect to bitcoind")
            })
    }
}

