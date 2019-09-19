mod network_graph;
mod lightning_address;
mod command;

use std::path::PathBuf;
use std::net::SocketAddr;
use std::sync::Arc;

use dependencies::httpbis;
use dependencies::tls_api_rustls;

use grpc::Client;
use httpbis::ClientTlsOption;
use structopt::StructOpt;
use tls_api_rustls::TlsConnector;

use build_info::get_build_info;

use self::command::{Command, Error};

#[derive(StructOpt, Debug)]
#[structopt(name = "lpd-client")]
/// Lightning Peach Client. Client for LPD (Lightning peach daemon)
pub struct Config {
    /// Address to connect
    #[structopt(long="addr", default_value="127.0.0.1:10009")]
    pub address: SocketAddr,

    /// Do not use TLS for rpc connections
    #[structopt(long="no-tls")]
    pub no_tls: bool,

    /// Path to cert file for TLS for RPC connections
    #[structopt(long="tls-cert-path", parse(from_os_str))]
    pub tls_cert_path: Option<PathBuf>,

    /// Print configuration information and exit. Useful for debugging
    #[structopt(long="print-config")]
    pub print_config: bool,

    /// Print detailed version
    #[structopt(long="version")]
    pub print_version: bool,

    #[structopt(subcommand)]
    pub command: Option<Command>,
}

fn print_version() {
    println!("{:#?}", get_build_info!());
}

fn main() -> Result<(), Error> {
    let config: Config = Config::from_args();
    if config.print_version {
        print_version();
        return Ok(());
    }
    if config.print_config {
        println!("{:#?}", &config);
        return Ok(());
    }
    if config.command.is_none() {
        eprintln!("Please specify a subcommand");
        std::process::exit(1);
    }
    let client = if !config.no_tls {
//        let cert_bytes = {
//            let mut file = File::open(cert_path)
//                .map_err(|err|
//                    Error::new_io_error(err, &format!("cannot open certificate file: {:?}", &cert_path))
//                )?;
//            let mut vec = Vec::new();
//            file.read_to_end(&mut vec).map_err(|err|
//                Error::new_io_error(err, &format!("cannot read certificate file: {:?}", &cert_path))
//            )?;
//            vec
//        };
        panic!("TLS is currently not implemented");
    } else {
        Client::new_expl::<TlsConnector>(
            &config.address,
            config.address.ip().to_string().as_str(),
            ClientTlsOption::Plain,
            Default::default()
        ).map_err(Error::Grpc)?
    };
    if let Some(command) = config.command {
        command.execute(Arc::new(client))
    } else {
        Ok(())
    }
}
