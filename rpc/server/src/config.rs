use tls_api_rustls::TlsAcceptor;
use tls_api::Error as TlsError;
use std::{net::{SocketAddr, AddrParseError}, io::Error as IoError};

#[derive(Debug)]
pub enum Error {
    Tls(TlsError),
    ReadCertificate(IoError),
    AddressParse(AddrParseError),
}

enum CommandLineKey {
    RpcAddress,
    P2pAddress,
    KeyFileName,
    CertFileName,
    DbPath,
}

impl CommandLineKey {
    fn pattern<'a>(&self) -> &'a str {
        use self::CommandLineKey::*;

        match self {
            RpcAddress => "--rpclisten=",
            P2pAddress => "--listen=",
            KeyFileName => "--key=",
            CertFileName => "--cert=",
            DbPath => "--db-path=",
        }
    }

    fn predicate(&self, arg: &String) -> bool {
        arg.starts_with(self.pattern())
    }

    fn value(&self, arg: String) -> String {
        arg.trim_start_matches(self.pattern()).to_owned()
    }
}

pub struct Argument {
    pub address: SocketAddr,
    pub p2p_address: SocketAddr,
    pub db_path: String,
    pub tls_acceptor: Option<TlsAcceptor>,
}

impl Argument {
    // TODO(mkl): rewite using some lib. All
    pub fn from_env() -> Result<Self, Error> {
        use tls_api_rustls::{
            TlsAcceptorBuilder as TlsAcceptorBuilderImpl,
        };
        use tls_api::TlsAcceptorBuilder;
        use std::{env, fs::File, io::Read, net::{Ipv4Addr, IpAddr}};
        use self::Error::*;
        use self::CommandLineKey::*;

        let default_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10009);
        let address = env::args()
            .find(|arg| RpcAddress.predicate(arg))
            .map(|arg| RpcAddress.value(arg).parse::<SocketAddr>())
            .unwrap_or(Ok(default_address))
            .map_err(AddressParse)?;

        let default_p2p_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9735);
        let p2p_address = env::args()
            .find(|arg| P2pAddress.predicate(arg))
            .map(|arg| P2pAddress.value(arg).parse::<SocketAddr>())
            .unwrap_or(Ok(default_p2p_address))
            .map_err(AddressParse)?;

        let default_db_path = "target/db";
        let db_path = env::args()
            .find(|arg| DbPath.predicate(arg))
            .map(|arg| DbPath.value(arg))
            .unwrap_or(default_db_path.to_owned());

        let acceptor = {
            let key_bytes: Option<std::io::Result<Vec<u8>>> = env::args().find(|arg| KeyFileName.predicate(arg)).map(|arg| {
                let path = KeyFileName.value(arg);
                let mut file = File::open(path)?;
                let mut vec = Vec::new();
                file.read_to_end(&mut vec)?;
                Ok(vec)
            });
            let cert_bytes = env::args().find(|arg| CertFileName.predicate(arg)).map(|arg| {
                let path = CertFileName.value(arg);
                let mut file = File::open(path)?;
                let mut vec = Vec::new();
                file.read_to_end(&mut vec)?;
                Ok(vec)
            });
            match (key_bytes, cert_bytes) {
                (Some(key_bytes), Some(cert_bytes)) => {
                    let cert_bytes = cert_bytes.map_err(ReadCertificate)?;
                    let acceptor =
                        TlsAcceptorBuilderImpl::from_certs_and_key(&[&cert_bytes[..]], &cert_bytes[..])
                            .map_err(Tls)?
                            .build()
                            .map_err(Tls)?;
                    Some(acceptor)
                }
                _ => None,
            }
        };

        Ok(Argument {
            address: address,
            p2p_address: p2p_address,
            db_path: db_path,
            tls_acceptor: acceptor,
        })
    }
}
