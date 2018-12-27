use grpc::{rt::ServerServiceDefinition, RequestOptions, SingleResponse};
use grpc::Error;
use interface::routing_grpc::{RoutingServiceServer, RoutingService};
use interface::routing::{
    SignMessageRequest, SignMessageResponse,
    ConnectPeerRequest, PeerList, Info,
    ChannelGraphRequest, ChannelGraph,
    QueryRoutesRequest, RouteList,
};
use interface::common::Void;
use connection::{Node, Command, AbstractAddress};
use std::sync::{RwLock, Arc};
use std::net::SocketAddr;
use futures::sync::mpsc::Sender;

pub fn service(node: Arc<RwLock<Node>>, control: Sender<Command<SocketAddr>>) -> ServerServiceDefinition {
    RoutingServiceServer::new_service_def(RoutingImpl {
        node: node,
        control: control,
    })
}

struct RoutingImpl<A>
where
    A: AbstractAddress,
{
    node: Arc<RwLock<Node>>,
    control: Sender<Command<A>>,
}

impl RoutingService for RoutingImpl<SocketAddr> {
    fn sign_message(&self, o: RequestOptions, p: SignMessageRequest) -> SingleResponse<SignMessageResponse> {
        let _ = (o, p);
        unimplemented!()
    }

    fn connect_peer(&self, o: RequestOptions, p: ConnectPeerRequest) -> SingleResponse<Void> {
        use secp256k1::PublicKey;
        use futures::{Sink, Future};

        let _ = o;

        let lightning_address = p.address.unwrap();
        let pk = hex::decode(lightning_address.pubkey.as_bytes()).unwrap();

        let command = Command::Connect {
            address: lightning_address.host.parse().unwrap(),
            remote_public: PublicKey::from_slice(pk.as_slice()).unwrap(),
        };

        let response = Void::new();

        let response = self.control.clone()
            .send(command)
            .map(|_| response)
            .map_err(|e| Error::Panic(format!("{:?}", e)));

        SingleResponse::no_metadata(response)
    }

    fn list_peers(&self, o: RequestOptions, p: Void) -> SingleResponse<PeerList> {
        let _ = (o, p);
        unimplemented!()
    }

    fn get_info(&self, o: RequestOptions, p: Void) -> SingleResponse<Info> {
        let _ = (o, p);
        unimplemented!()
    }

    fn describe_graph(&self, o: RequestOptions, p: ChannelGraphRequest) -> SingleResponse<ChannelGraph> {
        /*let state = self.state.read().unwrap();
        let _ = o;

        let (e, n) = state.describe(p.get_include_unannounced());

        let mut r = ChannelGraph::new();
        r.set_edges(e.into());
        r.set_nodes(n.into());
        SingleResponse::completed(r)*/
        let _ = (o, p);
        unimplemented!()
    }

    fn query_routes(&self, o: RequestOptions, p: QueryRoutesRequest) -> SingleResponse<RouteList> {
        let _ = (o, p);
        unimplemented!()
    }
}
