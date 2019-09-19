use std::net::SocketAddr;
use std::str::FromStr;
use dependencies::hex;

// TODO(mkl): allow usage of domain names
#[derive(Debug)]
pub struct LightningAddress {
    pub pub_key: String,
    pub host: SocketAddr,
}

impl FromStr for LightningAddress {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("@").collect();
        if parts.len() !=2 {
            return Err(format!("Lightning address is expected in format: pubkey@host:port, got {}", s))
        }
        let pub_key = parts[0];
        if let Ok(pk) = hex::decode(pub_key) {
            if pk.len() != 33 {
                return Err(format!("Wrong length of public key. Got {}, want {}", pk.len(), 33))
            }
        } else {
            return Err(format!("Cannot decode hex-encoded public key"));
        }
        let addr = SocketAddr::from_str(parts[1]).map_err(|err|{
            format!("cannot parse socket address: {} : {:?}", parts[1], err)
        })?;
        Ok(LightningAddress {
            pub_key: pub_key.to_owned(),
            host: addr,
        })
    }
}
