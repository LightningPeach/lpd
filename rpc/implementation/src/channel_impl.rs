use grpc::{rt::ServerServiceDefinition, RequestOptions, SingleResponse, StreamingResponse};
use interface::channel_grpc::{ChannelServiceServer, ChannelService};
use interface::channel::{
    ChannelFilter, ChannelList, PendingChannelsResponse,
    OpenChannelRequest, OpenStatusUpdate, CloseChannelRequest, CloseStatusUpdate,
};
use interface::common::Void;

pub fn service() -> ServerServiceDefinition {
    ChannelServiceServer::new_service_def(ChannelImpl)
}

struct ChannelImpl;

impl ChannelService for ChannelImpl {
    fn list(&self, o: RequestOptions, p: ChannelFilter) -> SingleResponse<ChannelList> {
        let _ = (o, p);
        unimplemented!()
    }

    fn pending(&self, o: RequestOptions, p: Void) -> SingleResponse<PendingChannelsResponse> {
        let _ = (o, p);
        unimplemented!()
    }

    fn open(&self, o: RequestOptions, p: OpenChannelRequest) -> StreamingResponse<OpenStatusUpdate> {
        let _ = (o, p);
        unimplemented!()
    }

    fn close(&self, o: RequestOptions, p: CloseChannelRequest) -> StreamingResponse<CloseStatusUpdate> {
        let _ = (o, p);
        unimplemented!()
    }
}
