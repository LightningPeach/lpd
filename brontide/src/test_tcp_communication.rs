use super::tcp_communication::{Listener, Stream, NetAddress};
use std::error::Error;
use std::thread;

use secp256k1::{Secp256k1, SecretKey, PublicKey};
use secp256k1::constants::SECRET_KEY_SIZE;
use rand;
use crossbeam;

fn make_listener() -> Result<(Listener, NetAddress), Box<Error>> {
    // First, generate the long-term private keys for the brontide listener.
    let local_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
    let local_priv = SecretKey::from_slice(&Secp256k1::new(), &local_priv_bytes)?;

    // Having a port of ":0" means a random port, and interface will be
	// chosen for our listener.
	let addr = String::from("localhost:0");

    // Our listener will be local, and the connection remote.
    let listener = Listener::new(local_priv, addr)?;

    let net_addr = NetAddress{
        identity_key: PublicKey::from_secret_key(&Secp256k1::new(), &local_priv)?,
        address: listener.local_addr()?,
    };

    Ok((listener, net_addr))
}

fn establish_test_connection() -> Result<(Stream, Stream), Box<Error>> {
	let (listener, net_addr) = make_listener()?;

	// Nos, generate the long-term private keys remote end of the connection
	// within our test.
    let remote_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
    let remote_priv = SecretKey::from_slice(&Secp256k1::new(), &remote_priv_bytes)?;

    // Initiate a connection with a separate goroutine, and listen with our
	// main one. If both errors are nil, then encryption+auth was
	// successful.
    let mut local_stream: Option<Stream> = None;
    let mut remote_stream: Option<Stream> = None;
    crossbeam::scope(|scope|{
        scope.spawn(|| {
            local_stream = Some(Stream::dial(remote_priv, net_addr).unwrap());
        });
        scope.spawn(|| {
            let tcp_stream = listener.accept().unwrap();
            remote_stream = Some(tcp_stream);
        });
    });

    Ok((local_stream.unwrap(), remote_stream.unwrap()))
}

#[test]
fn test_connection_correctness() {
	// Create a test connection, grabbing either side of the connection
	// into local variables. If the initial crypto handshake fails, then
	// we'll get a non-nil error here.
    let (mut local_conn, mut remote_conn) = establish_test_connection().unwrap();

    // Test out some message full-message reads.
	for i in 0..10 {
        let msg = format!("hello{}", i);
        let msg = msg.as_bytes();
        local_conn.encrypt_and_write_message(msg).unwrap();
        let vec: Vec<u8> = remote_conn.read_and_decrypt_message().unwrap();
        assert_eq!(vec.as_slice(), msg);
	}
//
//	// Now try incremental message reads. This simulates first writing a
//	// message header, then a message body.
//	outMsg := []byte("hello world")
//	if _, err := localConn.Write(outMsg); err != nil {
//		t.Fatalf("remote conn failed to write: %v", err)
//	}
//
//	readBuf := make([]byte, len(outMsg))
//	if _, err := remoteConn.Read(readBuf[:len(outMsg)/2]); err != nil {
//		t.Fatalf("local conn failed to read: %v", err)
//	}
//	if _, err := remoteConn.Read(readBuf[len(outMsg)/2:]); err != nil {
//		t.Fatalf("local conn failed to read: %v", err)
//	}
//
//	if !bytes.Equal(outMsg, readBuf) {
//		t.Fatalf("messages don't match, %v vs %v",
//			string(readBuf), string(outMsg))
//	}
}