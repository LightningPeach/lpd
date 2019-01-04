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
use std::fmt::Debug;
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

fn error<E>(e: E) -> Error where E: Debug {
    Error::Panic(format!("{:?}", e))
}

impl RoutingService for RoutingImpl<SocketAddr> {
    fn sign_message(&self, o: RequestOptions, p: SignMessageRequest) -> SingleResponse<SignMessageResponse> {
        use futures::future::err;
        use std::string::ToString;

        let _ = o;

        match Node::sign_message(self.node.clone(), p.message).map_err(error) {
            Ok(data) => {
                let mut response = SignMessageResponse::new();
                response.signature = data.to_string();
                SingleResponse::completed(response)
            }
            Err(e) => SingleResponse::no_metadata(err(e))
        }
    }

    fn connect_peer(&self, o: RequestOptions, p: ConnectPeerRequest) -> SingleResponse<Void> {
        use futures::{Sink, Future, future::err};

        let _ = o;

        fn connect_command(request: ConnectPeerRequest) -> Result<Command<SocketAddr>, Error> {
            use secp256k1::PublicKey;

            let mut request = request;
            let mut lightning_address = request.take_address();
            let pk = lightning_address.take_pubkey();
            let pk = hex::decode(pk.as_bytes()).map_err(error)?;
            let remote_public = PublicKey::from_slice(pk.as_slice()).map_err(error)?;
            let address = lightning_address.take_host().parse().map_err(error)?;

            Ok(Command::Connect {
                address: address,
                remote_public: remote_public,
            })
        }

        match connect_command(p) {
            Ok(command) => {
                let response = self.control.clone()
                    .send(command)
                    .map(|_| Void::new())
                    .map_err(error);
                SingleResponse::no_metadata(response)
            },
            Err(e) => SingleResponse::no_metadata(err(e)),
        }
    }

    fn list_peers(&self, o: RequestOptions, p: Void) -> SingleResponse<PeerList> {
        use std::string::ToString;
        use interface::routing::Peer;

        let _ = (o, p);

        let peers = Node::list_peers(self.node.clone())
            .iter()
            .map(ToString::to_string)
            .map(|pub_key| {
                let mut peer = Peer::new();
                peer.set_pub_key(pub_key);
                peer
            })
            .collect();

        let mut response = PeerList::new();
        response.set_peers(peers);
        SingleResponse::completed(response)
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
