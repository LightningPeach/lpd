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

pub trait RoutingService {
    fn sign_message(&self, o: ::grpc::RequestOptions, p: super::routing::SignMessageRequest) -> ::grpc::SingleResponse<super::routing::SignMessageResponse>;

    fn connect_peer(&self, o: ::grpc::RequestOptions, p: super::routing::ConnectPeerRequest) -> ::grpc::SingleResponse<super::common::Void>;

    fn list_peers(&self, o: ::grpc::RequestOptions, p: super::common::Void) -> ::grpc::SingleResponse<super::routing::PeerList>;

    fn get_info(&self, o: ::grpc::RequestOptions, p: super::common::Void) -> ::grpc::SingleResponse<super::routing::Info>;

    fn describe_graph(&self, o: ::grpc::RequestOptions, p: super::routing::ChannelGraphRequest) -> ::grpc::SingleResponse<super::routing::ChannelGraph>;

    fn describe_graph_dot_format(&self, o: ::grpc::RequestOptions, p: super::routing::ChannelGraphRequest) -> ::grpc::SingleResponse<super::routing::ChannelGraphDotFormat>;

    fn query_routes(&self, o: ::grpc::RequestOptions, p: super::routing::QueryRoutesRequest) -> ::grpc::SingleResponse<super::routing::RouteList>;
}

// client

pub struct RoutingServiceClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_SignMessage: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::routing::SignMessageRequest, super::routing::SignMessageResponse>>,
    method_ConnectPeer: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::routing::ConnectPeerRequest, super::common::Void>>,
    method_ListPeers: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::common::Void, super::routing::PeerList>>,
    method_GetInfo: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::common::Void, super::routing::Info>>,
    method_DescribeGraph: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::routing::ChannelGraphRequest, super::routing::ChannelGraph>>,
    method_DescribeGraphDotFormat: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::routing::ChannelGraphRequest, super::routing::ChannelGraphDotFormat>>,
    method_QueryRoutes: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::routing::QueryRoutesRequest, super::routing::RouteList>>,
}

impl ::grpc::ClientStub for RoutingServiceClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        RoutingServiceClient {
            grpc_client: grpc_client,
            method_SignMessage: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/RoutingService/SignMessage".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ConnectPeer: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/RoutingService/ConnectPeer".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ListPeers: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/RoutingService/ListPeers".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetInfo: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/RoutingService/GetInfo".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_DescribeGraph: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/RoutingService/DescribeGraph".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_DescribeGraphDotFormat: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/RoutingService/DescribeGraphDotFormat".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_QueryRoutes: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/RoutingService/QueryRoutes".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl RoutingService for RoutingServiceClient {
    fn sign_message(&self, o: ::grpc::RequestOptions, p: super::routing::SignMessageRequest) -> ::grpc::SingleResponse<super::routing::SignMessageResponse> {
        self.grpc_client.call_unary(o, p, self.method_SignMessage.clone())
    }

    fn connect_peer(&self, o: ::grpc::RequestOptions, p: super::routing::ConnectPeerRequest) -> ::grpc::SingleResponse<super::common::Void> {
        self.grpc_client.call_unary(o, p, self.method_ConnectPeer.clone())
    }

    fn list_peers(&self, o: ::grpc::RequestOptions, p: super::common::Void) -> ::grpc::SingleResponse<super::routing::PeerList> {
        self.grpc_client.call_unary(o, p, self.method_ListPeers.clone())
    }

    fn get_info(&self, o: ::grpc::RequestOptions, p: super::common::Void) -> ::grpc::SingleResponse<super::routing::Info> {
        self.grpc_client.call_unary(o, p, self.method_GetInfo.clone())
    }

    fn describe_graph(&self, o: ::grpc::RequestOptions, p: super::routing::ChannelGraphRequest) -> ::grpc::SingleResponse<super::routing::ChannelGraph> {
        self.grpc_client.call_unary(o, p, self.method_DescribeGraph.clone())
    }

    fn describe_graph_dot_format(&self, o: ::grpc::RequestOptions, p: super::routing::ChannelGraphRequest) -> ::grpc::SingleResponse<super::routing::ChannelGraphDotFormat> {
        self.grpc_client.call_unary(o, p, self.method_DescribeGraphDotFormat.clone())
    }

    fn query_routes(&self, o: ::grpc::RequestOptions, p: super::routing::QueryRoutesRequest) -> ::grpc::SingleResponse<super::routing::RouteList> {
        self.grpc_client.call_unary(o, p, self.method_QueryRoutes.clone())
    }
}

// server

pub struct RoutingServiceServer;


impl RoutingServiceServer {
    pub fn new_service_def<H : RoutingService + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/RoutingService",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/RoutingService/SignMessage".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.sign_message(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/RoutingService/ConnectPeer".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.connect_peer(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/RoutingService/ListPeers".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.list_peers(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/RoutingService/GetInfo".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_info(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/RoutingService/DescribeGraph".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.describe_graph(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/RoutingService/DescribeGraphDotFormat".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.describe_graph_dot_format(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/RoutingService/QueryRoutes".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.query_routes(o, p))
                    },
                ),
            ],
        )
    }
}
