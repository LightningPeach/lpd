use dependencies::grpc;
use dependencies::futures;
use dependencies::secp256k1;

use grpc::{rt::ServerServiceDefinition, RequestOptions, SingleResponse, StreamingResponse};
use grpc::Error;
use interface::channel_grpc::{ChannelServiceServer, ChannelService};
use interface::channel::{
    ChannelFilter, ChannelList, PendingChannelsResponse,
    OpenChannelRequest, OpenStatusUpdate, CloseChannelRequest, CloseStatusUpdate,
};
use interface::common::Void;
use connection::{AbstractAddress, Command};
use connection::Node;
use std::sync::{RwLock, Arc};
use std::net::SocketAddr;
use std::fmt::Debug;
use futures::sync::mpsc::Sender;
use secp256k1::PublicKey;
use internal_event::{DirectCommand, ChannelCommand};

pub fn service(node: Arc<RwLock<Node>>, control: Sender<Command<SocketAddr>>) -> ServerServiceDefinition {
    ChannelServiceServer::new_service_def(ChannelImpl {
        node: node,
        control: control,
    })
}

struct ChannelImpl<A>
where
    A: AbstractAddress,
{
    node: Arc<RwLock<Node>>,
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
        use futures::{Sink, Future, future, stream::Stream};
        use connection::ChannelStatus;
        use interface::channel::ChannelOpenUpdate;

        let _ = o;

        fn open_channel_command(request: OpenChannelRequest) -> Result<(Command<SocketAddr>, PublicKey), Error> {
            let pk = PublicKey::from_slice(request.get_node_pubkey()).map_err(error)?;
            let command = Command::DirectCommand {
                destination: pk.clone(),
                command: DirectCommand::NewChannel,
            };
            Ok((command, pk))
        }

        match open_channel_command(p) {
            Err(e) => StreamingResponse::no_metadata(future::err(e).into_stream()),
            Ok((command, public_key)) => {
                let receiver = self.node.write().unwrap()
                    .take_channel_status_stream(&public_key).expect("peer is not connected yet");
                let stream = self.control.clone()
                    .send(command)
                    .map_err(error)
                    .map(|_| receiver.map_err(|()| Error::Panic("Cannot send channel update".to_owned())))
                    .into_stream()
                    .flatten()
                    .map(|status: ChannelStatus| {
                        let mut response = OpenStatusUpdate::new();
                        match status {
                            ChannelStatus::Open => {
                                let mut update = ChannelOpenUpdate::new();
                                update.set_channel_point(unimplemented!());
                                response.set_chan_open(update)
                            },
                            _ => unimplemented!(),
                        };
                        response
                    });
                StreamingResponse::no_metadata(stream)
            }
        }
    }

    fn close(&self, o: RequestOptions, p: CloseChannelRequest) -> StreamingResponse<CloseStatusUpdate> {
        use futures::{Sink, Future, future, stream::Stream};
        use wire::ChannelId;

        let _ = o;

        fn channel_id(request: CloseChannelRequest) -> Result<ChannelId, Error> {

            let mut request = request;
            let mut point = request.take_channel_point();
            let mut channel_id = point.take_funding_txid_bytes();
            if channel_id.is_empty() {
                channel_id = point.take_funding_txid_str().into_bytes();
            }
            if channel_id.len() != 32 {
                return Err(Error::Panic("wrong size of founding txid".to_owned()))
            }
            let mut data = [0; 32];
            data.copy_from_slice(channel_id.as_slice());
            Ok(ChannelId {
                data: data,
            })
        }

        match channel_id(p) {
            Err(e) => StreamingResponse::no_metadata(future::err(e).into_stream()),
            Ok(channel_id) => {
                let command = Command::<SocketAddr>::ChannelCommand {
                    destination: channel_id,
                    command: ChannelCommand::CloseChannel,
                };

                unimplemented!()
            }
        }
    }
}
