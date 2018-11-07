extern crate bitcoin;
extern crate zmq;
extern crate futures;
extern crate tokio_core;

use bitcoin::{
    network::serialize::deserialize,
    util::hash::Sha256dHash,
    Block, Transaction, BitcoinHash, OutPoint,
};
use futures::{Poll, Async, Stream};

use std::sync::mpsc::{self, Sender, Receiver};

pub static DEFAULT_ZMQ_ADDR: &'static str = "tcp://localhost:18501";
pub static DEFAULT_RPC_ADDR: &'static str = "http://localhost:18443";
pub static DEFAULT_RPC_USER: &'static str = "user";
pub static DEFAULT_RPC_PASS: &'static str = "password";

#[derive(Debug)]
pub enum ZMQMessageType {
    RawBlock,
    RawTx,
}

impl<'a> From<&'a str> for ZMQMessageType {
    fn from(value: &str) -> Self {
        match value {
            "rawblock" => ZMQMessageType::RawBlock,
            "rawtx"    => ZMQMessageType::RawTx,
            _          => unreachable!(),
        }
    }
}
impl From<ZMQMessageType> for String {
    fn from(msg_type: ZMQMessageType) -> String {
        match msg_type {
            ZMQMessageType::RawBlock => "rawblock".to_string(),
            ZMQMessageType::RawTx    => "rawtx".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum ZMQMessage {
    Block(Block),
    Tx(Transaction),
}

pub struct ZMQMessageProducer {
    socket: zmq::Socket,
}

impl ZMQMessageProducer {
    pub fn new() -> Self {
        println!("connecting to bitcoind's server...");
        let context = zmq::Context::new();
        let socket = context.socket(zmq::SUB).unwrap();
        socket.set_subscribe(String::from(ZMQMessageType::RawBlock).as_bytes()).unwrap();
        socket.set_subscribe(String::from(ZMQMessageType::RawTx).as_bytes()).unwrap();
        assert!(socket.connect(DEFAULT_ZMQ_ADDR).is_ok());
        Self { socket }
    }
}

impl Stream for ZMQMessageProducer {
    type Item = ZMQMessage;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let poll_item = self.socket.as_poll_item(zmq::POLLIN);
        match zmq::poll(&mut [poll_item], 0).unwrap() {
            0 => {
                futures::task::current().notify();
                Ok(Async::NotReady)
            },
            _ => {
                let msg_type = ZMQMessageType::from(self.socket.recv_string(0).unwrap().unwrap().as_str());
                let bytes = self.socket.recv_bytes(0).unwrap();
                let msg = match msg_type {
                    ZMQMessageType::RawBlock => {
                        let block: Block = deserialize(&bytes).unwrap();
                        ZMQMessage::Block(block)

                    },
                    ZMQMessageType::RawTx => {
                        let tx: Transaction = deserialize(&bytes).unwrap();
                        ZMQMessage::Tx(tx)
                    },
                };
                self.socket.recv_string(0).unwrap().unwrap().as_str();
                Ok(Async::Ready(Some(msg)))
            }
        }
    }
}

#[derive(Debug)]
pub enum ConfirmationEvent {
    Mempool(ConfirmationEventMempool),
    Confirmed(ConfirmationEventConfirmed),
}

#[derive(Debug)]
pub struct ConfirmationEventMempool {
    txid: Sha256dHash,
}

#[derive(Debug)]
pub struct ConfirmationEventConfirmed {
    txid: Sha256dHash,
    block_hash: Sha256dHash,
}

#[derive(Debug)]
pub enum SpendEvent {
    Mempool(SpendEventMempool),
    Confirmed(SpendEventConfirmed),
}
#[derive(Debug)]
pub struct SpendEventMempool {
    out_point: OutPoint,
    txid: Sha256dHash,
}
#[derive(Debug)]
pub struct SpendEventConfirmed {
    out_point: OutPoint,
    txid: Sha256dHash,
    block_hash: Sha256dHash,
}

struct ConfirmationSubscription {
    pub txid: Sha256dHash,
    pub num_confs: u8,
    sender: Sender<ConfirmationEvent>,
}

struct SpentSubscription {
    pub out_point: OutPoint,
    sender: Sender<SpendEvent>,
}

pub struct ZMQMessageConsumer {
    confirmation_subscriptions: Vec<ConfirmationSubscription>,
    spent_subscriptions: Vec<SpentSubscription>,
    rx: Receiver<ZMQMessage>,
}

impl ZMQMessageConsumer {
    pub fn new(rx: Receiver<ZMQMessage>) -> Self {
        Self {
            confirmation_subscriptions: Vec::new(),
            spent_subscriptions: Vec::new(),
            rx,
        }
    }

    pub fn register_confirmations_ntfn(
        &mut self,
        txid: Sha256dHash,
        num_confs: u8,
    ) -> Receiver<ConfirmationEvent> {
        let (sender, receiver): (Sender<ConfirmationEvent>, Receiver<ConfirmationEvent>) = mpsc::channel();
        self.confirmation_subscriptions.push(ConfirmationSubscription{
            txid,
            num_confs,
            sender,
        });
        receiver
    }

    pub fn register_spend_ntfn(
        &mut self,
        out_point: OutPoint,
    ) -> Receiver<SpendEvent> {
        let (sender, receiver): (Sender<SpendEvent>, Receiver<SpendEvent>) = mpsc::channel();
        self.spent_subscriptions.push(SpentSubscription{
            out_point,
            sender,
        });
        receiver
    }
}

impl Stream for ZMQMessageConsumer {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.rx.try_recv() {
            // TODO(evg): match TryRecvError
            Err(_) => {
                futures::task::current().notify();
                Ok(Async::NotReady)
            },
            Ok(message) => {
                match message {
                    ZMQMessage::Block(block) => {
                        for confirmation_subscription in &self.confirmation_subscriptions {
                            for tx in &block.txdata {
                                if tx.txid() == confirmation_subscription.txid {
                                    let event = ConfirmationEventConfirmed {
                                        txid: confirmation_subscription.txid,
                                        block_hash: block.bitcoin_hash(),
                                    };
                                    confirmation_subscription.sender.send(ConfirmationEvent::Confirmed(event)).unwrap();
                                }
                            }
                        }
                        for spent_subscription in &self.spent_subscriptions {
                            for tx in &block.txdata {
                                for input in &tx.input {
                                    if input.previous_output == spent_subscription.out_point {
                                        let event = SpendEventConfirmed {
                                            out_point: input.previous_output,
                                            txid: tx.txid(),
                                            block_hash: block.bitcoin_hash(),
                                        };
                                        spent_subscription.sender.send(SpendEvent::Confirmed(event)).unwrap();
                                    }
                                }
                            }
                        }
                    },
                    ZMQMessage::Tx(tx) => {
                        for confirmation_subscription in &self.confirmation_subscriptions {
                            if tx.txid() == confirmation_subscription.txid {
                                let event = ConfirmationEventMempool {
                                    txid: confirmation_subscription.txid,
                                };
                                confirmation_subscription.sender.send(ConfirmationEvent::Mempool(event)).unwrap();
                            }
                        }
                        for spent_subscription in &self.spent_subscriptions {
                            for input in &tx.input {
                                if input.previous_output == spent_subscription.out_point {
                                    let event = SpendEventMempool {
                                        out_point: input.previous_output,
                                        txid: tx.txid(),
                                    };
                                    spent_subscription.sender.send(SpendEvent::Mempool(event)).unwrap();
                                }
                            }
                        }
                    },
                }
                Ok(Async::Ready(Some(())))
            }
        }
    }
}

pub struct FutureConfirmationEvent {
    rx: Receiver<ConfirmationEvent>,
}

impl FutureConfirmationEvent {
    pub fn new(rx: Receiver<ConfirmationEvent>) -> Self {
        Self { rx }
    }
}

impl Stream for FutureConfirmationEvent {
    type Item = ConfirmationEvent;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.rx.try_recv() {
            // TODO(evg): match TryRecvError
            Err(_) => {
                futures::task::current().notify();
                Ok(Async::NotReady)
            },
            Ok(msg) => {
                Ok(Async::Ready(Some(msg)))
            }
        }
    }
}

pub struct FutureSpendEvent {
    rx: Receiver<SpendEvent>,
}

impl FutureSpendEvent {
    pub fn new(rx: Receiver<SpendEvent>) -> Self {
        Self { rx }
    }
}

impl Stream for FutureSpendEvent {
    type Item = SpendEvent;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.rx.try_recv() {
            // TODO(evg): match TryRecvError
            Err(_) => {
                futures::task::current().notify();
                Ok(Async::NotReady)
            },
            Ok(msg) => {
                Ok(Async::Ready(Some(msg)))
            }
        }
    }
}