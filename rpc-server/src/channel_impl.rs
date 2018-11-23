use grpc::{rt::ServerServiceDefinition, RequestOptions, SingleResponse, StreamingResponse};
use super::channel_grpc::{ChannelServiceServer, ChannelService};
use super::channel::{
    ChannelFilter, ChannelList, PendingChannelsResponse,
    OpenChannelRequest, OpenStatusUpdate, CloseChannelRequest, CloseStatusUpdate,
};
use super::common::{Void, Satoshi as ProtoSatoshi, MilliSatoshi as ProtoMilliSatoshi};

use wire::{Wrapper, Satoshi, MilliSatoshi};

impl From<ProtoSatoshi> for Satoshi {
    fn from(v: ProtoSatoshi) -> Self {
        Satoshi::default().fmap(|_| v.value)
    }
}

impl From<ProtoMilliSatoshi> for MilliSatoshi {
    fn from(v: ProtoMilliSatoshi) -> Self {
        MilliSatoshi::default().fmap(|_| v.value)
    }
}

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
