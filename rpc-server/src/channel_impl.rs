use grpc::{rt::ServerServiceDefinition, RequestOptions, SingleResponse};
use super::channel_grpc::{ChannelServiceServer, ChannelService};
use super::channel::{ChannelList, Void};

pub fn service() -> ServerServiceDefinition {
    ChannelServiceServer::new_service_def(ChannelImpl)
}

struct ChannelImpl;

impl ChannelService for ChannelImpl {
    fn list(&self, o: RequestOptions, p: Void) -> SingleResponse<ChannelList> {
        let _ = (o, p);
        unimplemented!()
    }

    fn pending(&self, o: RequestOptions, p: Void) -> SingleResponse<Void> {
        let _ = (o, p);
        unimplemented!()
    }

    fn open(&self, o: RequestOptions, p: Void) -> SingleResponse<Void> {
        let _ = (o, p);
        unimplemented!()
    }

    fn close(&self, o: RequestOptions, p: Void) -> SingleResponse<Void> {
        let _ = (o, p);
        unimplemented!()
    }
}
