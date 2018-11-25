use grpc::{rt::ServerServiceDefinition, RequestOptions, SingleResponse, StreamingRequest, StreamingResponse};
use super::payment_grpc::{PaymentServiceServer, PaymentService};
use super::payment::{
    SendRequest, SendResponse, Invoice, AddInvoiceResponse,
    ListInvoiceRequest, ListInvoiceResponse, PayReqString, PayReq, PaymentList
};
use super::common::Void;

pub fn service() -> ServerServiceDefinition {
    PaymentServiceServer::new_service_def(PaymentImpl)
}

struct PaymentImpl;

impl PaymentService for PaymentImpl {
    fn send_payment(&self, o: RequestOptions, p: StreamingRequest<SendRequest>) -> StreamingResponse<SendResponse> {
        let _ = (o, p);
        unimplemented!()
    }

    fn send_payment_sync(&self, o: RequestOptions, p: SendRequest) -> SingleResponse<SendResponse> {
        let _ = (o, p);
        unimplemented!()
    }

    fn add_invoice(&self, o: RequestOptions, p: Invoice) -> SingleResponse<AddInvoiceResponse> {
        let _ = (o, p);
        unimplemented!()
    }

    fn list_invoices(&self, o: RequestOptions, p: ListInvoiceRequest) -> SingleResponse<ListInvoiceResponse> {
        let _ = (o, p);
        unimplemented!()
    }

    fn decode_pay_req(&self, o: RequestOptions, p: PayReqString) -> SingleResponse<PayReq> {
        let _ = (o, p);
        unimplemented!()
    }

    fn list_payments(&self, o: RequestOptions, p: Void) -> SingleResponse<PaymentList> {
        let _ = (o, p);
        unimplemented!()
    }
}
