// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

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

pub trait PaymentService {
    fn send_payment(&self, o: ::grpc::RequestOptions, p: ::grpc::StreamingRequest<super::payment::SendRequest>) -> ::grpc::StreamingResponse<super::payment::SendResponse>;

    fn send_payment_sync(&self, o: ::grpc::RequestOptions, p: super::payment::SendRequest) -> ::grpc::SingleResponse<super::payment::SendResponse>;

    fn add_invoice(&self, o: ::grpc::RequestOptions, p: super::payment::Invoice) -> ::grpc::SingleResponse<super::payment::AddInvoiceResponse>;

    fn list_invoices(&self, o: ::grpc::RequestOptions, p: super::payment::ListInvoiceRequest) -> ::grpc::SingleResponse<super::payment::ListInvoiceResponse>;

    fn decode_pay_req(&self, o: ::grpc::RequestOptions, p: super::payment::PayReqString) -> ::grpc::SingleResponse<super::payment::PayReq>;

    fn list_payments(&self, o: ::grpc::RequestOptions, p: super::common::Void) -> ::grpc::SingleResponse<super::payment::PaymentList>;
}

// client

pub struct PaymentServiceClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_SendPayment: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::payment::SendRequest, super::payment::SendResponse>>,
    method_SendPaymentSync: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::payment::SendRequest, super::payment::SendResponse>>,
    method_AddInvoice: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::payment::Invoice, super::payment::AddInvoiceResponse>>,
    method_ListInvoices: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::payment::ListInvoiceRequest, super::payment::ListInvoiceResponse>>,
    method_DecodePayReq: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::payment::PayReqString, super::payment::PayReq>>,
    method_ListPayments: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::common::Void, super::payment::PaymentList>>,
}

impl ::grpc::ClientStub for PaymentServiceClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        PaymentServiceClient {
            grpc_client: grpc_client,
            method_SendPayment: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/PaymentService/SendPayment".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Bidi,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_SendPaymentSync: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/PaymentService/SendPaymentSync".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_AddInvoice: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/PaymentService/AddInvoice".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ListInvoices: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/PaymentService/ListInvoices".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_DecodePayReq: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/PaymentService/DecodePayReq".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ListPayments: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/PaymentService/ListPayments".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl PaymentService for PaymentServiceClient {
    fn send_payment(&self, o: ::grpc::RequestOptions, p: ::grpc::StreamingRequest<super::payment::SendRequest>) -> ::grpc::StreamingResponse<super::payment::SendResponse> {
        self.grpc_client.call_bidi(o, p, self.method_SendPayment.clone())
    }

    fn send_payment_sync(&self, o: ::grpc::RequestOptions, p: super::payment::SendRequest) -> ::grpc::SingleResponse<super::payment::SendResponse> {
        self.grpc_client.call_unary(o, p, self.method_SendPaymentSync.clone())
    }

    fn add_invoice(&self, o: ::grpc::RequestOptions, p: super::payment::Invoice) -> ::grpc::SingleResponse<super::payment::AddInvoiceResponse> {
        self.grpc_client.call_unary(o, p, self.method_AddInvoice.clone())
    }

    fn list_invoices(&self, o: ::grpc::RequestOptions, p: super::payment::ListInvoiceRequest) -> ::grpc::SingleResponse<super::payment::ListInvoiceResponse> {
        self.grpc_client.call_unary(o, p, self.method_ListInvoices.clone())
    }

    fn decode_pay_req(&self, o: ::grpc::RequestOptions, p: super::payment::PayReqString) -> ::grpc::SingleResponse<super::payment::PayReq> {
        self.grpc_client.call_unary(o, p, self.method_DecodePayReq.clone())
    }

    fn list_payments(&self, o: ::grpc::RequestOptions, p: super::common::Void) -> ::grpc::SingleResponse<super::payment::PaymentList> {
        self.grpc_client.call_unary(o, p, self.method_ListPayments.clone())
    }
}

// server

pub struct PaymentServiceServer;


impl PaymentServiceServer {
    pub fn new_service_def<H : PaymentService + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/PaymentService",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/PaymentService/SendPayment".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Bidi,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerBidi::new(move |o, p| handler_copy.send_payment(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/PaymentService/SendPaymentSync".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.send_payment_sync(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/PaymentService/AddInvoice".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.add_invoice(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/PaymentService/ListInvoices".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.list_invoices(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/PaymentService/DecodePayReq".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.decode_pay_req(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/PaymentService/ListPayments".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.list_payments(o, p))
                    },
                ),
            ],
        )
    }
}
