use dependencies::grpc;
use dependencies::futures;
use dependencies::hex;
use dependencies::secp256k1;

use grpc::{rt::ServerServiceDefinition, RequestOptions, SingleResponse};
use grpc::Error;
use interface::routing_grpc::{RoutingServiceServer, RoutingService};
use interface::routing::{SignMessageRequest, SignMessageResponse, ConnectPeerRequest, PeerList, Info, ChannelGraphRequest, ChannelGraph, ChannelGraphDotFormat, QueryRoutesRequest, RouteList};
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

impl<A> RoutingImpl<A>
where
    A: AbstractAddress,
{
    pub fn graph(&self, include_unannounced: bool) -> ChannelGraph {
        let (edges, nodes) = self.node.read().unwrap().describe_graph(include_unannounced);

        let mut graph = ChannelGraph::new();
        graph.set_edges(edges.into());
        graph.set_nodes(nodes.into());
        graph
    }
}

fn error<E>(e: E) -> Error where E: Debug {
    Error::Panic(format!("{:?}", e))
}

impl RoutingService for RoutingImpl<SocketAddr> {
    fn sign_message(&self, o: RequestOptions, p: SignMessageRequest) -> SingleResponse<SignMessageResponse> {
        use std::string::ToString;

        let _ = o;

        let signature = self.node.read().unwrap().sign_message(p.message);
        let mut response = SignMessageResponse::new();
        response.signature = signature.to_string();
        SingleResponse::completed(response)
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

        let peers = self.node.read().unwrap().list_peers()
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

        let response = self.node.write().unwrap().get_info();

        SingleResponse::completed(response)
    }

    fn describe_graph(&self, o: RequestOptions, p: ChannelGraphRequest) -> SingleResponse<ChannelGraph> {
        let _ = o;

        let include_unannounced = p.get_include_unannounced();
        let graph = self.graph(include_unannounced);

        SingleResponse::completed(graph)
    }

    fn describe_graph_dot_format(&self, o: RequestOptions, p: ChannelGraphRequest) -> SingleResponse<ChannelGraphDotFormat> {
        use interface::routing::{LightningNode, ChannelEdge};

        let _ = o;

        let include_unannounced = p.get_include_unannounced();
        let graph = self.graph(include_unannounced);

        let nodes_number = graph.nodes.len();
        let edges_number = graph.edges.len();

        let node_dot = |node: &LightningNode| -> String {
            let addresses = node.addresses.iter()
                .fold(String::new(), |a, address| format!("{};{} {}", a, address.network, address.addr));
            format!(
                "{} [alias=\"{}\",last_update={},color=\"{}\",addresses=\"{}\"];\n",
                node.pub_key,
                node.alias,
                node.last_update,
                node.color,
                addresses.trim_start_matches(';'),
            )
        };

        let edge_dot = |edge: &ChannelEdge| -> String {
            use interface::{common::MilliSatoshi, routing::RoutingPolicy};

            let policy = |policy: Option<&RoutingPolicy>| -> String {
                if let Some(policy) = policy {
                    format!(
                        "{{time_lock_delta: {}, min_htlc: {}, fee_base: {}, fee_rate: {}, disabled: {}}}",
                        policy.time_lock_delta,
                        policy.min_htlc,
                        policy.fee_base_msat,
                        policy.fee_rate_milli.as_ref().map(MilliSatoshi::get_value).unwrap_or(0),
                        policy.disabled,
                    )
                } else {
                    "None".to_owned()
                }
            };
            format!(
                "{} -- {} [last_update={},capacity={},channel_id={:x},chan_point=\"{}\",node1_policy=\"{}\",node2_policy=\"{}\"]\n",
                edge.node1_pub,
                edge.node2_pub,
                edge.last_update,
                edge.capacity,
                edge.channel_id,
                edge.chan_point,
                policy(edge.node1_policy.as_ref()),
                policy(edge.node2_policy.as_ref()),
            )
        };

        let nodes = graph.nodes.iter().fold(String::new(), |a, item| a + node_dot(item).as_str());
        let edges = graph.edges.iter().fold(String::new(), |a, item| a + edge_dot(item).as_str());

        let graph_dot_description = format!(
            "graph network_map {{\n\
            \tsize = \"{},{}\";\n\
            \t{}\
            \t{}\
            }}\
            ",
            nodes_number,
            edges_number,
            nodes,
            edges,
        );

        let mut response = ChannelGraphDotFormat::new();
        response.set_raw(graph_dot_description);

        SingleResponse::completed(response)
    }

    fn query_routes(&self, o: RequestOptions, p: QueryRoutesRequest) -> SingleResponse<RouteList> {
        use futures::future::err;
        use secp256k1::PublicKey;

        let _ = o;

        fn parse_input(request: QueryRoutesRequest) -> Result<PublicKey, Error> {
            let mut request = request;
            let pk = request.take_pub_key();
            let pk = hex::decode(pk.as_bytes()).map_err(error)?;
            PublicKey::from_slice(pk.as_slice()).map_err(error)
        }

        match parse_input(p) {
            Ok(goal) => {
                use interface::common::{Route, Hop};

                let v = self.node.read().unwrap().find_route(goal);

                let hops = v.into_iter()
                    .map(|(mut node, channel)| {
                        let mut hop = Hop::new();
                        hop.set_chan_id(channel.get_channel_id());
                        hop.set_chan_capacity(channel.get_capacity());
                        hop.set_fee_msat(channel.get_node1_policy().get_fee_base_msat());
                        hop.set_pub_key(node.take_pub_key());
                        hop
                    }).collect();

                let mut route = Route::new();
                route.set_hops(hops);

                let list = vec![route].into();
                let mut response = RouteList::new();
                response.set_routes(list);

                SingleResponse::completed(response)
            },
            Err(e) => SingleResponse::no_metadata(err(e)),
        }
    }
}
