use grpc::{Client, ClientStub, Error};
use std::{sync::Arc, net::SocketAddr};

use interface::{
    routing_grpc::{RoutingServiceClient, RoutingService},
    channel_grpc::{ChannelServiceClient, ChannelService},
    payment_grpc::{PaymentServiceClient, PaymentService},
};

pub struct LightningPeach(Arc<Client>);

impl LightningPeach {
    pub fn new(socket_address: &SocketAddr) -> Result<Self, Error> {
        use tls_api_stub::TlsConnector;
        use httpbis::ClientTlsOption;

        let host = socket_address.ip().to_string();
        let client = Client::new_expl::<TlsConnector>(
            &socket_address,
            host.as_str(),
            ClientTlsOption::Plain,
            Default::default()
        )?;

        Ok(LightningPeach(Arc::new(client)))
    }

    pub fn local(port: u16) -> Result<Self, Error> {
        use std::net::{SocketAddrV4, Ipv4Addr};

        // TODO(mkl): make it configurable
        let socket_address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port));
        Self::new(&socket_address)
    }

    pub fn routing(&self) -> impl RoutingService {
        RoutingServiceClient::with_client(self.0.clone())
    }

    pub fn channel(&self) -> impl ChannelService {
        ChannelServiceClient::with_client(self.0.clone())
    }

    pub fn payment(&self) -> impl PaymentService {
        PaymentServiceClient::with_client(self.0.clone())
    }
}
