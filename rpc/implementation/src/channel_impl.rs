use grpc::{rt::ServerServiceDefinition, RequestOptions, SingleResponse, StreamingResponse};
use grpc::Error;
use interface::channel_grpc::{ChannelServiceServer, ChannelService};
use interface::channel::{
    ChannelFilter, ChannelList, PendingChannelsResponse,
    OpenChannelRequest, OpenStatusUpdate, CloseChannelRequest, CloseStatusUpdate,
};
use interface::common::Void;
use connection::{AbstractAddress, Command};
use std::net::SocketAddr;
use std::fmt::Debug;
use futures::sync::mpsc::Sender;
use secp256k1::PublicKey;
use internal_event::DirectCommand;

pub fn service(control: Sender<Command<SocketAddr>>) -> ServerServiceDefinition {
    ChannelServiceServer::new_service_def(ChannelImpl {
        control: control,
    })
}

struct ChannelImpl<A>
where
    A: AbstractAddress,
{
    control: Sender<Command<A>>,
}

fn error<E>(e: E) -> Error where E: Debug {
    Error::Panic(format!("{:?}", e))
}

impl ChannelService for ChannelImpl<SocketAddr> {
    fn list(&self, o: RequestOptions, p: ChannelFilter) -> SingleResponse<ChannelList> {
        let _ = (o, p);
        unimplemented!()
    }

    fn pending(&self, o: RequestOptions, p: Void) -> SingleResponse<PendingChannelsResponse> {
        let _ = (o, p);
        unimplemented!()
    }

    fn open(&self, o: RequestOptions, p: OpenChannelRequest) -> StreamingResponse<OpenStatusUpdate> {
        use futures::{Sink, Future, future, stream};

        let _ = o;

        fn open_channel_command(request: OpenChannelRequest) -> Result<Command<SocketAddr>, Error> {
            let pk = PublicKey::from_slice(request.get_node_pubkey()).map_err(error)?;
            Ok(Command::DirectCommand {
                destination: pk,
                command: DirectCommand::NewChannel,
            })
        }

        match open_channel_command(p) {
            Err(e) => StreamingResponse::no_metadata(future::err(e).into_stream()),
            Ok(command) => {
                let future = self.control.clone()
                    .send(command)
                    .map(|_| unimplemented!())
                    .map_err(error);
                StreamingResponse::no_metadata(future.into_stream())
            }
        }
    }

    fn close(&self, o: RequestOptions, p: CloseChannelRequest) -> StreamingResponse<CloseStatusUpdate> {
        let _ = (o, p);
        unimplemented!()
    }
}
