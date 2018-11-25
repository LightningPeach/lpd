use grpc::{rt::ServerServiceDefinition, RequestOptions, SingleResponse};
use super::routing_grpc::{RoutingServiceServer, RoutingService};
use super::routing::{
    SignMessageRequest, SignMessageResponse,
    ConnectPeerRequest, PeerList, Info,
    ChannelGraphRequest, ChannelGraph,
    QueryRoutesRequest, RouteList,
};
use super::common::Void;

pub fn service() -> ServerServiceDefinition {
    RoutingServiceServer::new_service_def(RoutingImpl)
}

struct RoutingImpl;

impl RoutingService for RoutingImpl {
    fn sign_message(&self, o: RequestOptions, p: SignMessageRequest) -> SingleResponse<SignMessageResponse> {
        let _ = (o, p);
        unimplemented!()
    }

    fn connect_peer(&self, o: RequestOptions, p: ConnectPeerRequest) -> SingleResponse<Void> {
        let _ = (o, p);
        unimplemented!()
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
        let _ = (o, p);
        unimplemented!()
    }

    fn query_routes(&self, o: RequestOptions, p: QueryRoutesRequest) -> SingleResponse<RouteList> {
        let _ = (o, p);
        unimplemented!()
    }
}