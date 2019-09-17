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

pub trait Wallet {
    fn new_address(&self, o: ::grpc::RequestOptions, p: super::wallet::NewAddressRequest) -> ::grpc::SingleResponse<super::wallet::NewAddressResponse>;

    fn new_change_address(&self, o: ::grpc::RequestOptions, p: super::wallet::NewChangeAddressRequest) -> ::grpc::SingleResponse<super::wallet::NewChangeAddressResponse>;

    fn get_utxo_list(&self, o: ::grpc::RequestOptions, p: super::wallet::GetUtxoListRequest) -> ::grpc::SingleResponse<super::wallet::GetUtxoListResponse>;

    fn wallet_balance(&self, o: ::grpc::RequestOptions, p: super::wallet::WalletBalanceRequest) -> ::grpc::SingleResponse<super::wallet::WalletBalanceResponse>;

    fn sync_with_tip(&self, o: ::grpc::RequestOptions, p: super::wallet::SyncWithTipRequest) -> ::grpc::SingleResponse<super::wallet::SyncWithTipResponse>;

    fn make_tx(&self, o: ::grpc::RequestOptions, p: super::wallet::MakeTxRequest) -> ::grpc::SingleResponse<super::wallet::MakeTxResponse>;

    fn send_coins(&self, o: ::grpc::RequestOptions, p: super::wallet::SendCoinsRequest) -> ::grpc::SingleResponse<super::wallet::SendCoinsResponse>;

    fn unlock_coins(&self, o: ::grpc::RequestOptions, p: super::wallet::UnlockCoinsRequest) -> ::grpc::SingleResponse<super::wallet::UnlockCoinsResponse>;

    fn shutdown(&self, o: ::grpc::RequestOptions, p: super::wallet::ShutdownRequest) -> ::grpc::SingleResponse<super::wallet::ShutdownResponse>;
}

// client

pub struct WalletClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_NewAddress: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wallet::NewAddressRequest, super::wallet::NewAddressResponse>>,
    method_NewChangeAddress: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wallet::NewChangeAddressRequest, super::wallet::NewChangeAddressResponse>>,
    method_GetUtxoList: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wallet::GetUtxoListRequest, super::wallet::GetUtxoListResponse>>,
    method_WalletBalance: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wallet::WalletBalanceRequest, super::wallet::WalletBalanceResponse>>,
    method_SyncWithTip: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wallet::SyncWithTipRequest, super::wallet::SyncWithTipResponse>>,
    method_MakeTx: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wallet::MakeTxRequest, super::wallet::MakeTxResponse>>,
    method_SendCoins: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wallet::SendCoinsRequest, super::wallet::SendCoinsResponse>>,
    method_UnlockCoins: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wallet::UnlockCoinsRequest, super::wallet::UnlockCoinsResponse>>,
    method_Shutdown: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::wallet::ShutdownRequest, super::wallet::ShutdownResponse>>,
}

impl ::grpc::ClientStub for WalletClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        WalletClient {
            grpc_client: grpc_client,
            method_NewAddress: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/walletrpc.Wallet/NewAddress".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_NewChangeAddress: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/walletrpc.Wallet/NewChangeAddress".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetUtxoList: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/walletrpc.Wallet/GetUtxoList".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_WalletBalance: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/walletrpc.Wallet/WalletBalance".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_SyncWithTip: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/walletrpc.Wallet/SyncWithTip".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_MakeTx: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/walletrpc.Wallet/MakeTx".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_SendCoins: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/walletrpc.Wallet/SendCoins".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UnlockCoins: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/walletrpc.Wallet/UnlockCoins".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Shutdown: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/walletrpc.Wallet/Shutdown".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl Wallet for WalletClient {
    fn new_address(&self, o: ::grpc::RequestOptions, p: super::wallet::NewAddressRequest) -> ::grpc::SingleResponse<super::wallet::NewAddressResponse> {
        self.grpc_client.call_unary(o, p, self.method_NewAddress.clone())
    }

    fn new_change_address(&self, o: ::grpc::RequestOptions, p: super::wallet::NewChangeAddressRequest) -> ::grpc::SingleResponse<super::wallet::NewChangeAddressResponse> {
        self.grpc_client.call_unary(o, p, self.method_NewChangeAddress.clone())
    }

    fn get_utxo_list(&self, o: ::grpc::RequestOptions, p: super::wallet::GetUtxoListRequest) -> ::grpc::SingleResponse<super::wallet::GetUtxoListResponse> {
        self.grpc_client.call_unary(o, p, self.method_GetUtxoList.clone())
    }

    fn wallet_balance(&self, o: ::grpc::RequestOptions, p: super::wallet::WalletBalanceRequest) -> ::grpc::SingleResponse<super::wallet::WalletBalanceResponse> {
        self.grpc_client.call_unary(o, p, self.method_WalletBalance.clone())
    }

    fn sync_with_tip(&self, o: ::grpc::RequestOptions, p: super::wallet::SyncWithTipRequest) -> ::grpc::SingleResponse<super::wallet::SyncWithTipResponse> {
        self.grpc_client.call_unary(o, p, self.method_SyncWithTip.clone())
    }

    fn make_tx(&self, o: ::grpc::RequestOptions, p: super::wallet::MakeTxRequest) -> ::grpc::SingleResponse<super::wallet::MakeTxResponse> {
        self.grpc_client.call_unary(o, p, self.method_MakeTx.clone())
    }

    fn send_coins(&self, o: ::grpc::RequestOptions, p: super::wallet::SendCoinsRequest) -> ::grpc::SingleResponse<super::wallet::SendCoinsResponse> {
        self.grpc_client.call_unary(o, p, self.method_SendCoins.clone())
    }

    fn unlock_coins(&self, o: ::grpc::RequestOptions, p: super::wallet::UnlockCoinsRequest) -> ::grpc::SingleResponse<super::wallet::UnlockCoinsResponse> {
        self.grpc_client.call_unary(o, p, self.method_UnlockCoins.clone())
    }

    fn shutdown(&self, o: ::grpc::RequestOptions, p: super::wallet::ShutdownRequest) -> ::grpc::SingleResponse<super::wallet::ShutdownResponse> {
        self.grpc_client.call_unary(o, p, self.method_Shutdown.clone())
    }
}

// server

pub struct WalletServer;


impl WalletServer {
    pub fn new_service_def<H : Wallet + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/walletrpc.Wallet",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/walletrpc.Wallet/NewAddress".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.new_address(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/walletrpc.Wallet/NewChangeAddress".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.new_change_address(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/walletrpc.Wallet/GetUtxoList".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_utxo_list(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/walletrpc.Wallet/WalletBalance".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.wallet_balance(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/walletrpc.Wallet/SyncWithTip".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.sync_with_tip(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/walletrpc.Wallet/MakeTx".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.make_tx(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/walletrpc.Wallet/SendCoins".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.send_coins(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/walletrpc.Wallet/UnlockCoins".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.unlock_coins(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/walletrpc.Wallet/Shutdown".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.shutdown(o, p))
                    },
                ),
            ],
        )
    }
}
