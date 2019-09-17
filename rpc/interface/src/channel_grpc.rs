// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait ChannelService {
    fn list(&self, o: ::grpc::RequestOptions, p: super::channel::ChannelFilter) -> ::grpc::SingleResponse<super::channel::ChannelList>;

    fn pending(&self, o: ::grpc::RequestOptions, p: super::common::Void) -> ::grpc::SingleResponse<super::channel::PendingChannelsResponse>;

    fn open(&self, o: ::grpc::RequestOptions, p: super::channel::OpenChannelRequest) -> ::grpc::StreamingResponse<super::channel::OpenStatusUpdate>;

    fn close(&self, o: ::grpc::RequestOptions, p: super::channel::CloseChannelRequest) -> ::grpc::StreamingResponse<super::channel::CloseStatusUpdate>;
}

// client

pub struct ChannelServiceClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_List: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::channel::ChannelFilter, super::channel::ChannelList>>,
    method_Pending: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::common::Void, super::channel::PendingChannelsResponse>>,
    method_Open: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::channel::OpenChannelRequest, super::channel::OpenStatusUpdate>>,
    method_Close: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::channel::CloseChannelRequest, super::channel::CloseStatusUpdate>>,
}

impl ::grpc::ClientStub for ChannelServiceClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        ChannelServiceClient {
            grpc_client: grpc_client,
            method_List: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/ChannelService/List".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Pending: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/ChannelService/Pending".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Open: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/ChannelService/Open".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Close: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/ChannelService/Close".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl ChannelService for ChannelServiceClient {
    fn list(&self, o: ::grpc::RequestOptions, p: super::channel::ChannelFilter) -> ::grpc::SingleResponse<super::channel::ChannelList> {
        self.grpc_client.call_unary(o, p, self.method_List.clone())
    }

    fn pending(&self, o: ::grpc::RequestOptions, p: super::common::Void) -> ::grpc::SingleResponse<super::channel::PendingChannelsResponse> {
        self.grpc_client.call_unary(o, p, self.method_Pending.clone())
    }

    fn open(&self, o: ::grpc::RequestOptions, p: super::channel::OpenChannelRequest) -> ::grpc::StreamingResponse<super::channel::OpenStatusUpdate> {
        self.grpc_client.call_server_streaming(o, p, self.method_Open.clone())
    }

    fn close(&self, o: ::grpc::RequestOptions, p: super::channel::CloseChannelRequest) -> ::grpc::StreamingResponse<super::channel::CloseStatusUpdate> {
        self.grpc_client.call_server_streaming(o, p, self.method_Close.clone())
    }
}

// server

pub struct ChannelServiceServer;


impl ChannelServiceServer {
    pub fn new_service_def<H : ChannelService + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/ChannelService",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/ChannelService/List".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.list(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/ChannelService/Pending".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.pending(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/ChannelService/Open".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerServerStreaming::new(move |o, p| handler_copy.open(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/ChannelService/Close".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerServerStreaming::new(move |o, p| handler_copy.close(o, p))
                    },
                ),
            ],
        )
    }
}
