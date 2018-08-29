use secp256k1::{SecretKey, PublicKey};
use std::net::{TcpListener, TcpStream, SocketAddr, ToSocketAddrs};
use std::io::{self, Read, Write};
use super::{Machine, ACT_ONE_SIZE, ACT_TWO_SIZE, ACT_THREE_SIZE, MachineRead, MachineWrite};
use super::HandshakeError;
use super::BrontideError;
use super::IoError;

// Listener is an implementation of a net.Conn which executes an authenticated
// key exchange and message encryption protocol dubbed "Machine" after
// initial connection acceptance. See the Machine struct for additional
// details w.r.t the handshake and encryption scheme used within the
// connection.
pub struct Listener {
    local_static: SecretKey,
    tcp: TcpListener,
}

impl Listener {
    // new returns a new net.Listener which enforces the Brontide scheme
    // during both initial connection establishment and data transfer.
    pub fn new<A>(local_static: SecretKey, listen_addr: A) -> Result<Self, IoError> where A: ToSocketAddrs {
        // TODO(evg): call something like golang's net.ResolveTCPAddr("tcp", listenAddr)

        let listener = TcpListener::bind(listen_addr)?;

        let brontide_listener = Listener {
            local_static: local_static,
            tcp: listener,
        };

        // TODO(evg): call brontide_listener.listen()

        Ok(brontide_listener)
    }

    pub fn accept(&self) -> Result<Stream, HandshakeError> {
        let (tcp_stream, _socket_address) = self.tcp.accept().map_err(HandshakeError::Io)?;

        self.do_handshake(tcp_stream)
    }

    // doHandshake asynchronously performs the brontide handshake, so that it does
    // not block the main accept loop. This prevents peers that delay writing to the
    // connection from block other connection attempts.
    pub fn do_handshake(&self, tcp_stream: TcpStream) -> Result<Stream, HandshakeError> {
//        defer func() { l.handshakeSema <- struct{}{} }()
//
//        select {
//        case <-l.quit:
//            return
//        default:
//        }
//
        // Initialize machine with empty remote public key. Due third act of handshake we
        // receive(from remote peer) and implicitly verify one.
        let mut brontide_stream = Stream::new(
            tcp_stream,
            Machine::new(false, self.local_static, PublicKey::new(), &[])
                .map_err(HandshakeError::Crypto)?,
        );
//
//        // We'll ensure that we get ActOne from the remote peer in a timely
//        // manner. If they don't respond within 1s, then we'll kill the
//        // connection.
//        conn.SetReadDeadline(time.Now().Add(handshakeReadTimeout))
//
        // Attempt to carry out the first act of the handshake protocol. If the
        // connecting node doesn't know our long-term static public key, then
        // this portion will fail with a non-nil error.
        let mut act_one: [u8; ACT_ONE_SIZE] = [0; ACT_ONE_SIZE];
        brontide_stream.read_exact(&mut act_one[..])
            .map_err(HandshakeError::Io)?;

        brontide_stream.noise.recv_act_one(act_one)?;

        // Next, progress the handshake processes by sending over our ephemeral
        // key for the session along with an authenticating tag.
        let act_two = brontide_stream.noise.gen_act_two()?;
        brontide_stream.write_all(&act_two[..])
            .map_err(HandshakeError::Io)?;

//
//        // We'll ensure that we get ActTwo from the remote peer in a timely
//        // manner. If they don't respond within 1 second, then we'll kill the
//        // connection.
//        conn.SetReadDeadline(time.Now().Add(handshakeReadTimeout))
//
        // Finally, finish the handshake processes by reading and decrypting
        // the connection peer's static public key. If this succeeds then both
        // sides have mutually authenticated each other.
        let mut act_three: [u8; ACT_THREE_SIZE] = [0; ACT_THREE_SIZE];
        brontide_stream.read_exact(&mut act_three[..])
            .map_err(HandshakeError::Io)?;

        brontide_stream.noise.recv_act_three(act_three)?;

//
//        // We'll reset the deadline as it's no longer critical beyond the
//        // initial handshake.
//        conn.SetReadDeadline(time.Time{})

        Ok(brontide_stream)
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.tcp.local_addr()
    }
}

// Stream is an implementation of net.Conn which enforces an authenticated key
// exchange and message encryption protocol dubbed "Brontide" after initial TCP
// connection establishment. In the case of a successful handshake, all
// messages sent via the .Write() method are encrypted with an AEAD cipher
// along with an encrypted length-prefix. See the Machine struct for
// additional details w.r.t to the handshake and encryption scheme.
// TODO(evg): both fields must be private
pub struct Stream {
    stream: TcpStream,
    noise: Machine,
}

impl Stream {
    pub fn new(stream: TcpStream, noise: Machine) -> Self {
        Self {
            stream,
            noise,
        }
    }

    // dial attempts to establish an encrypted+authenticated connection with the
    // remote peer located at address which has remotePub as its long-term static
    // public key. In the case of a handshake failure, the connection is closed and
    // an error is returned.
    pub fn dial(local_priv: SecretKey, net_addr: NetAddress,
        /* dialer: fn(String, String)  -> Result<Stream, Box<Error>> */) -> Result<Stream, HandshakeError> {

        let stream = TcpStream::connect(net_addr.address)
            .map_err(HandshakeError::Io)?;
//        ipAddr := netAddr.Address.String()
//        var conn net.Conn
//        var err error
//        conn, err = dialer("tcp", ipAddr)
//        if err != nil {
//            return nil, err
//        }
//
        let mut brontide_stream = Stream{
            stream,
            noise: Machine::new(true, local_priv, net_addr.identity_key, &[])
                .map_err(HandshakeError::Crypto)?,
        };

        // Initiate the handshake by sending the first act to the receiver.
        let act_one = brontide_stream.noise.gen_act_one()?;

        brontide_stream.stream.write_all(&act_one[..])
            .map_err(HandshakeError::Io)?;
//
//        // We'll ensure that we get ActTwo from the remote peer in a timely
//        // manner. If they don't respond within 1s, then we'll kill the
//        // connection.
//        conn.SetReadDeadline(time.Now().Add(handshakeReadTimeout))
//
        // If the first act was successful (we know that address is actually
        // remotePub), then read the second act after which we'll be able to
        // send our static public key to the remote peer with strong forward
        // secrecy.
        let mut act_two: [u8; ACT_TWO_SIZE] = [0; ACT_TWO_SIZE];
        brontide_stream.stream.read_exact(&mut act_two[..])
            .map_err(HandshakeError::Io)?;

        brontide_stream.noise.recv_act_two(act_two)?;

        // Finally, complete the handshake by sending over our encrypted static
        // key and execute the final ECDH operation.
        let act_three = brontide_stream.noise.gen_act_three()?;

        brontide_stream.stream.write_all(&act_three[..])
            .map_err(HandshakeError::Io)?;
//
//        // We'll reset the deadline as it's no longer critical beyond the
//        // initial handshake.
//        conn.SetReadDeadline(time.Time{})
//
//        return b, nil
        Ok(brontide_stream)
    }

    pub fn encrypt_and_write_message(&mut self, msg: &[u8]) -> Result<(), BrontideError> {
        self.noise.write_message(&mut self.stream, msg)
    }

    pub fn read_and_decrypt_message(&mut self) -> Result<Vec<u8>, BrontideError> {
        self.noise.read_message(&mut self.stream)
    }

    pub fn as_read(&mut self) -> MachineRead<&mut TcpStream> {
        let &mut Stream {
            stream: ref mut stream,
            noise: ref mut noise,
        } = self;

        MachineRead {
            noise: noise,
            read: stream,
        }
    }

    pub fn as_write(&mut self) -> MachineWrite<&mut TcpStream> {
        let &mut Stream {
            stream: ref mut stream,
            noise: ref mut noise,
        } = self;

        MachineWrite {
            noise: noise,
            write: stream,
        }
    }
}

// TODO(evg): reimplement Read/Write traits, its implementation should
// automatically call decrypt/encrypt methods
impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

// NetAddress represents information pertaining to the identity and network
// reachability of a peer. Information stored includes the node's identity
// public key for establishing a confidential+authenticated connection, the
// service bits it supports, and a TCP address the node is reachable at.
pub struct NetAddress {
	// identity_key is the long-term static public key for a node. This node is
	// used throughout the network as a node's identity key. It is used to
	// authenticate any data sent to the network on behalf of the node, and
	// additionally to establish a confidential+authenticated connection with
	// the node.
	pub identity_key: PublicKey,

	// address is the IP address and port of the node. This is left
	// general so that multiple implementations can be used.
	pub address: SocketAddr,
}
