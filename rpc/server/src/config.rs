use dependencies::tls_api;
use dependencies::tls_api_rustls;

use tls_api_rustls::TlsAcceptor;
use tls_api::Error as TlsError;
use tls_api::TlsAcceptorBuilder;
use std::{net::{SocketAddr, AddrParseError}, io::Error as IoError};
use std::path::PathBuf;
use std::fs::File;

use structopt::StructOpt;
use std::io::Read;

use tls_api_rustls::{
    TlsAcceptorBuilder as TlsAcceptorBuilderImpl,
};

#[derive(StructOpt, Debug)]
#[structopt(name = "lpd")]
/// Lightning Peach Daemon. Server implementation of Lighting protocol
pub struct Config {
    /// Address to listen for RPC client connections
    #[structopt(long="rpc-listen", default_value="127.0.0.1:10009")]
    pub rpc_address: SocketAddr,

    /// Address to listen for peer-to-peer(p2p) connections
    #[structopt(long="p2p-listen", default_value="127.0.0.1:9735")]
    pub p2p_address: SocketAddr,

    /// Path to database
    #[structopt(long="db-path", parse(from_os_str), default_value="target/db")]
    pub db_path: PathBuf,

    /// Do not use TLS for rpc connections
    #[structopt(long="rpc-no-tls")]
    pub rpc_no_tls: bool,

    /// Path to key file for TLS for RPC connections
    #[structopt(long="rpc-tls-key-path", parse(from_os_str))]
    pub rpc_tls_key_path: Option<PathBuf>,

    /// Path to cert file for TLS for RPC connections
    #[structopt(long="rpc-tls-cert-path", parse(from_os_str))]
    pub rpc_tls_cert_path: Option<PathBuf>,

    /// Print configuration information and exit. Useful for debugging
    #[structopt(long="print-config")]
    pub print_config: bool,

    /// Print detailed version
    #[structopt(long="version")]
    pub print_version: bool,
}


#[derive(Debug)]
pub enum Error {
    Tls {
        inner: TlsError,
        description: String,
    },
    ReadCertificate{
        inner: IoError,
        description: String,
    }
}
impl Error {
    fn new_tls_error(inner: TlsError, description: &str) -> Error {
        Error::Tls {
            inner,
            description: description.to_owned(),
        }
    }

    fn new_read_certificate_error(inner: IoError, description: &str) -> Error {
        Error::ReadCertificate {
            inner,
            description: description.to_owned(),
        }
    }
}
pub fn create_tls_acceptor(cert_path: &PathBuf, key_path: &PathBuf) -> Result<TlsAcceptor, Error> {
    let cert_bytes = {
        let mut file = File::open(cert_path)
            .map_err(|err|
                Error::new_read_certificate_error(err, &format!("cannot open certificate file: {:?}", &cert_path))
            )?;
        let mut vec = Vec::new();
        file.read_to_end(&mut vec).map_err(|err|
            Error::new_read_certificate_error(err, &format!("cannot read certificate file: {:?}", &cert_path))
        )?;
        vec
    };
    let key_bytes = {
        let mut file = File::open(key_path)
            .map_err(|err|
                Error::new_read_certificate_error(err, &format!("cannot open key file: {:?}", &key_path))
            )?;
        let mut vec = Vec::new();
        file.read_to_end(&mut vec).map_err(|err|
            Error::new_read_certificate_error(err, &format!("cannot read key file: {:?}", &key_path))
        )?;
        vec
    };
    let acceptor =
        TlsAcceptorBuilderImpl::from_certs_and_key(&[&cert_bytes[..]], &key_bytes[..])
            .map_err(|err|{
                Error::new_tls_error(err, "cannot create TlsAcceptorBuilder")
            })?
            .build()
            .map_err(|err| {
                Error::new_tls_error(err, "cannot create TlsAcceptor")
            })?;
    Ok(acceptor)
}
