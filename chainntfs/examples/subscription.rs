use dependencies::zmq;
use dependencies::bitcoin_rpc_client;
use dependencies::futures;
use dependencies::tokio_core;
use dependencies::bitcoin;
use dependencies::bitcoin_hashes;

use bitcoin::{
    OutPoint
};
use bitcoin_hashes::sha256d;
use bitcoin_rpc_client::{
    Client, RpcApi, Auth
};

use futures::Stream;
use tokio_core::reactor::Core;

use std::sync::mpsc::{self, Sender, Receiver};

use chainntfs::{
    ZMQMessageProducer, ZMQMessageConsumer, ConfirmationEvent, SpendEvent,
    FutureConfirmationEvent, FutureSpendEvent, ZMQMessage,
    DEFAULT_RPC_ADDR, DEFAULT_RPC_USER, DEFAULT_RPC_PASS,
};

fn main() {
    let client = Client::new(
        DEFAULT_RPC_ADDR.to_owned(),
        Auth::UserPass(DEFAULT_RPC_USER.to_owned(), DEFAULT_RPC_PASS.to_owned())
    ).unwrap();
    let block_hashes = client.generate(110, None).unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let (sender, receiver): (Sender<ZMQMessage>, Receiver<ZMQMessage>) = mpsc::channel();
    let producer = ZMQMessageProducer::new()
        .for_each(|message| {
            match message {
                ZMQMessage::Block(block) => {
                    println!("block");
                    sender.send(ZMQMessage::Block(block)).unwrap();
                },
                ZMQMessage::Tx(tx) => {
                    println!("tx");
                    sender.send(ZMQMessage::Tx(tx)).unwrap();
                },
            };
            Ok(())
        });
    let mut consumer = ZMQMessageConsumer::new(receiver);

    let addr = client.get_new_address(None, None).unwrap();
    let txid = client.send_to_address(
        &addr,
        1.0,
        None,
        None,
        None,
        None,
        None,
        None,
    ).unwrap();
    let num_confs = 1;
    let conf_rx = consumer.register_confirmations_ntfn(
        txid,
        num_confs,
    );

    let mut spend_rx_vec = Vec::new();
    for block_hash in block_hashes {
        let block = client.get_block(&block_hash).unwrap();
        let coinbase_txid = block.txdata[0].txid();
        spend_rx_vec.push(consumer.register_spend_ntfn(
            OutPoint{
                txid: coinbase_txid,
                vout: 0,
            },
        ));
    }
    let consumer = consumer.for_each(|_| {
            Ok(())
        });
    handle.spawn(consumer);
    handle.spawn(FutureConfirmationEvent::new(conf_rx).for_each(|confirmation_event| {
        match confirmation_event {
            ConfirmationEvent::Mempool(event)   => println!("{:?}", event),
            ConfirmationEvent::Confirmed(event) => println!("{:?}", event),
        }
        Ok(())
    }));
    for spend_rx in spend_rx_vec {
        handle.spawn(FutureSpendEvent::new(spend_rx).for_each(|spend_event| {
            match spend_event {
                SpendEvent::Mempool(event)   => println!("{:?}", event),
                SpendEvent::Confirmed(event) => println!("{:?}", event),
            }
            Ok(())
        }));
    }

    core.run(producer).unwrap();
}