use grpc::{rt::ServerServiceDefinition, RequestOptions, SingleResponse};
use grpc::Error;
use interface::wallet_grpc::{WalletServer, Wallet};
use interface::wallet::{
    NewAddressRequest, NewAddressResponse, NewChangeAddressRequest, NewChangeAddressResponse,
    GetUtxoListRequest, GetUtxoListResponse, SyncWithTipRequest, SyncWithTipResponse,
    MakeTxRequest, MakeTxResponse, SendCoinsRequest, SendCoinsResponse,
    WalletBalanceRequest, WalletBalanceResponse, AddressType as RpcAddressType, Utxo as RpcUtxo,
    OutPoint as RpcOutPoint,
    UnlockCoinsRequest, UnlockCoinsResponse, ShutdownRequest, ShutdownResponse
};
use std::sync::{Mutex, Arc};
use futures::sync::mpsc::Sender;
use std::fmt::Debug;

use bitcoin::{
    network::serialize::serialize,
    blockdata::transaction::OutPoint,
};

use wallet_lib::{
    account::AccountAddressType,
    walletlibrary::LockId,
    interface::Wallet as WalletInterface,
};

pub fn service<A>(af: Arc<Mutex<Box<WalletInterface + Send>>>, shutdown: Sender<Command<A>>) -> ServerServiceDefinition
where
    A: AbstractAddress + Send + Sync + 'static,
{
    WalletServer::new_service_def(WalletImpl::new(af, shutdown))
}

fn error<E>(e: E) -> Error where E: Debug {
    Error::Panic(format!("{:?}", e))
}

fn grpc_result<T, E>(r: Result<T, E>) -> SingleResponse<T>
where
    T: Send,
    E: Debug,
{
    r
        .map_err(error)
        .map(SingleResponse::completed)
        .unwrap_or_else(SingleResponse::err)
}

use connection::{Command, AbstractAddress};

struct WalletImpl<A>
where
    A: AbstractAddress,
{
    af: Arc<Mutex<Box<WalletInterface + Send>>>,
    shutdown: Sender<Command<A>>,
}

impl<A> WalletImpl<A>
where
    A: AbstractAddress,
{
    fn new(af: Arc<Mutex<Box<WalletInterface + Send>>>, shutdown: Sender<Command<A>>) -> Self {
        WalletImpl {
            af: af,
            shutdown: shutdown,
        }
    }

    fn new_address_helper(&self, req: &NewAddressRequest) -> Result<NewAddressResponse, Box<dyn std::error::Error>> {
        let mut resp = NewAddressResponse::new();
        let mut ac = self.af.lock().unwrap();
        let address_type = req.get_addr_type();
        // TODO: implement From trait
        let address_type = match address_type {
            RpcAddressType::P2PKH  => AccountAddressType::P2PKH,
            RpcAddressType::P2SHWH => AccountAddressType::P2SHWH,
            RpcAddressType::P2WKH  => AccountAddressType::P2WKH,
        };
        let account = ac.wallet_lib_mut().get_account_mut(address_type);
        let addr = account.new_address()?;
        resp.set_address(addr);
        Ok(resp)
    }

    fn new_change_address(&self, req: &NewChangeAddressRequest) -> Result<NewChangeAddressResponse, Box<dyn std::error::Error>> {
        let mut resp = NewChangeAddressResponse::new();
        let mut ac = self.af.lock().unwrap();
        let address_type = req.get_addr_type();
        // TODO: implement From trait
        let address_type = match address_type {
            RpcAddressType::P2PKH  => AccountAddressType::P2PKH,
            RpcAddressType::P2SHWH => AccountAddressType::P2SHWH,
            RpcAddressType::P2WKH  => AccountAddressType::P2WKH,
        };
        let account = ac.wallet_lib_mut().get_account_mut(address_type);
        let addr = account.new_change_address()?;
        resp.set_address(addr);
        Ok(resp)
    }

    fn make_tx_helper(&self, req: MakeTxRequest) -> Result<MakeTxResponse, Box<dyn std::error::Error>> {
        let mut ops = Vec::new();
        for op in req.ops.into_vec() {
            ops.push(OutPoint {
                txid: From::from(op.txid.as_slice()),
                vout: op.vout,
            })
        }

        let tx = self.af.lock().unwrap().make_tx(ops, req.dest_addr, req.amt, req.submit)?;

        let mut resp = MakeTxResponse::new();
        resp.set_serialized_raw_tx(serialize(&tx)?);
        Ok(resp)
    }

    fn send_coins_helper(&self, req: SendCoinsRequest) -> Result<SendCoinsResponse, Box<dyn std::error::Error>> {
        let (tx, lock_id) = self.af.lock().unwrap().send_coins(req.dest_addr, req.amt, req.lock_coins, req.witness_only, req.submit)?;

        let mut resp = SendCoinsResponse::new();
        resp.set_serialized_raw_tx(serialize(&tx).unwrap());
        if req.lock_coins {
            resp.set_lock_id(lock_id.into());
        }
        Ok(resp)
    }
}

impl<A> Wallet for WalletImpl<A>
where
    A: AbstractAddress,
{
    fn new_address(&self, o: RequestOptions, p: NewAddressRequest) -> SingleResponse<NewAddressResponse> {
        let _ = o;
        grpc_result(self.new_address_helper(&p))
    }

    fn new_change_address(&self, o: RequestOptions, p: NewChangeAddressRequest) -> SingleResponse<NewChangeAddressResponse> {
        let _ = o;
        grpc_result(self.new_change_address(&p))
    }

    fn get_utxo_list(&self, o: RequestOptions, p: GetUtxoListRequest) -> SingleResponse<GetUtxoListResponse> {
        use protobuf::RepeatedField;

        let _ = (o, p);
        let mut resp = GetUtxoListResponse::new();
        let utxo_list = self.af.lock().unwrap().wallet_lib().get_utxo_list();
        resp.set_utxos(RepeatedField::from_vec(utxo_list.into_iter().map(|utxo| {
            let mut op = RpcOutPoint::new();
            op.set_txid(utxo.out_point.txid.into_bytes().to_vec());
            op.set_vout(utxo.out_point.vout);

            let mut rpc_utxo = RpcUtxo::new();
            rpc_utxo.set_value(utxo.value.into());
            rpc_utxo.set_out_point(op);
            let address_type = match utxo.addr_type {
                AccountAddressType::P2PKH  => RpcAddressType::P2PKH,
                AccountAddressType::P2SHWH => RpcAddressType::P2SHWH,
                AccountAddressType::P2WKH  => RpcAddressType::P2WKH,
            };
            rpc_utxo.set_addr_type(address_type);
            rpc_utxo
        }).collect()));
        grpc::SingleResponse::completed(resp)
    }

    fn wallet_balance(&self, o: RequestOptions, p: WalletBalanceRequest) -> SingleResponse<WalletBalanceResponse> {
        let _ = (o, p);

        let mut resp = WalletBalanceResponse::new();
        let balance = self.af.lock().unwrap().wallet_lib().wallet_balance();
        resp.set_total_balance(balance);
        SingleResponse::completed(resp)
    }

    fn sync_with_tip(&self, o: RequestOptions, p: SyncWithTipRequest) -> SingleResponse<SyncWithTipResponse> {
        let _ = (o, p);

        let resp = SyncWithTipResponse::new();
        self.af.lock().unwrap().sync_with_tip();
        SingleResponse::completed(resp)
    }

    fn make_tx(&self, o: RequestOptions, p: MakeTxRequest) -> SingleResponse<MakeTxResponse> {
        let _ = o;

        grpc_result(self.make_tx_helper(p))
    }

    fn send_coins(&self, o: RequestOptions, p: SendCoinsRequest) -> SingleResponse<SendCoinsResponse> {
        let _ = o;

        grpc_result(self.send_coins_helper(p))
    }

    fn unlock_coins(&self, o: RequestOptions, p: UnlockCoinsRequest) -> SingleResponse<UnlockCoinsResponse> {
        let _ = o;

        self.af.lock().unwrap().wallet_lib_mut().unlock_coins(LockId::from(p.lock_id));
        SingleResponse::completed(UnlockCoinsResponse::new())
    }

    fn shutdown(&self, o: RequestOptions, p: ShutdownRequest) -> SingleResponse<ShutdownResponse> {
        use futures::sink::Sink;
        use futures::future::Future;

        let _ = (o, p);

        self.shutdown.clone().send(Command::Terminate).wait().unwrap();
        SingleResponse::completed(ShutdownResponse::new())
    }
}
