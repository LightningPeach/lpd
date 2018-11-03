#[cfg(test)]
mod test_bolt0008;
mod async_stream;
mod serde;
mod cipher_state;
mod symmetric_state;
mod handshake;

pub use self::async_stream::BrontideStream;
pub use self::handshake::{HandshakeError, Machine};
use self::handshake::*;

/*impl fmt::Debug for HandshakeState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let remote_ephemeral_str = match self.remote_ephemeral {
            None => "None".to_owned(),
            Some(k) => hex::encode(&k.serialize()[..]),
        };

        write!(f, r#"
        symmetric_state: {:?}

        initiator: {:?}

        local_static:    {:?}
        local_ephemeral: {:?}

        remote_static:    {:?}
        remote_ephemeral: {:?}
        "#, self.symmetric_state, self.initiator,
               self.local_static, self.local_ephemeral,
               hex::encode(&self.remote_static.serialize()[..]),
               remote_ephemeral_str,
        )
    }
}*/

/*impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"
        send_cipher:     {:?}
        recv_cipher:     {:?}
        handshake_state: {:?}
        "#, self.send_cipher, self.recv_cipher, self.handshake_state,
        )
    }
}*/
