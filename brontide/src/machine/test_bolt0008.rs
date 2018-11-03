use secp256k1::{Secp256k1, SecretKey, PublicKey, Error as CryptoError};
use hex;
use std::error::Error;
use std::collections::HashMap;
use super::handshake::HandshakeNew;

#[test]
fn test_bolt0008() {
    test_bolt0008_internal().unwrap();
}

fn test_bolt0008_internal() -> Result<(), Box<Error>> {
    let rs_priv = SecretKey::from_slice(
        &Secp256k1::new(),
        hex::decode("2121212121212121212121212121212121212121212121212121212121212121")?.as_slice(),
    )?;
    let rs_pub = PublicKey::from_secret_key(&Secp256k1::new(), &rs_priv)?;
    assert_eq!(
        hex::encode(&rs_pub.serialize()[..]),
        "028d7500dd4c12685d1f568b4c2b5048e8534b873319f3a8daa612b469132ec7f7"
    );

    let ls_priv = SecretKey::from_slice(
        &Secp256k1::new(),
        hex::decode("1111111111111111111111111111111111111111111111111111111111111111")?.as_slice(),
    )?;
    let ls_pub = PublicKey::from_secret_key(&Secp256k1::new(), &ls_priv)?;
    assert_eq!(
        hex::encode(&ls_pub.serialize()[..]),
        "034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa"
    );

    let e_priv = SecretKey::from_slice(
        &Secp256k1::new(),
        hex::decode("1212121212121212121212121212121212121212121212121212121212121212")?.as_slice(),
    )?;
    let e_pub = PublicKey::from_secret_key(&Secp256k1::new(), &e_priv)?;
    assert_eq!(
        hex::encode(&e_pub.serialize()[..]),
        "036360e856310ce5d294e8be33fc807077dc56ac80d95d9cd4ddbd21325eff73f7"
    );

    let mut machine = HandshakeNew::new(true, ls_priv, rs_pub)?;
    machine.ephemeral_gen = || -> Result<SecretKey, CryptoError> {
        let sk = SecretKey::from_slice(
            &Secp256k1::new(),
            hex::decode("1212121212121212121212121212121212121212121212121212121212121212")
                .unwrap()
                .as_slice(),
        )?;
        Ok(sk)
    };
    assert_eq!(
        hex::encode(machine.handshake_digest()),
        "8401b3fdcaaa710b5405400536a3d5fd7792fe8e7fe29cd8b687216fe323ecbd"
    );

    let (act_one, machine) = machine.gen_act_one()?;

    assert_eq!(hex::encode(&act_one.bytes[..]), "00036360e856310ce5d294e8be33fc807077dc56ac80d95d9cd4ddbd21325eff73f70df6086551151f58b8afe6c195782c6a");

    let mut responder_machine = HandshakeNew::new(false, rs_priv, ls_pub)?;
    responder_machine.ephemeral_gen = || -> Result<SecretKey, CryptoError> {
        let sk = SecretKey::from_slice(
            &Secp256k1::new(),
            hex::decode("2222222222222222222222222222222222222222222222222222222222222222")
                .unwrap()
                .as_slice(),
        )?;
        Ok(sk)
    };

    let responder_machine = responder_machine.recv_act_one(act_one)?;

    let (act_two, responder_machine) = responder_machine.gen_act_two()?;
    assert_eq!(hex::encode(&act_two.bytes[..]), "0002466d7fcae563e5cb09a0d1870bb580344804617879a14949cf22285f1bae3f276e2470b93aac583c9ef6eafca3f730ae");

    let machine = machine.recv_act_two(act_two)?;

    let (act_three, mut machine) = machine.gen_act_three()?;
    assert_eq!(hex::encode(&act_three.bytes[..]), "00b9e3a702e93e3a9948c2ed6e5fd7590a6e1c3a0344cfc9d5b57357049aa22355361aa02e55a8fc28fef5bd6d71ad0c38228dc68b1c466263b47fdf31e560e139ba");

    let mut responder_machine = responder_machine.recv_act_three(act_three)?;

    println!("{:?}", responder_machine);

    let send_key = "969ab31b4d288cedf6218839b27a3e2140827047f2c0f01bf5c04435d43511a9";
    let recv_key = "bb9020b8965f4df047e07f955f3c4b88418984aadc5cdb35096b9ea8fa5c3442";
    let chain_key = "919219dbb2920afa8db80f9a51787a840bcf111ed8d588caf9ab4be716e42b01";

    assert_eq!(hex::encode(&machine.send_cipher_key()[..]), send_key);
    assert_eq!(hex::encode(&machine.recv_cipher_key()[..]), recv_key);
    assert_eq!(hex::encode(&machine.chaining_key()[..]), chain_key);

    assert_eq!(
        hex::encode(&responder_machine.send_cipher_key()[..]),
        recv_key
    );
    assert_eq!(
        hex::encode(&responder_machine.recv_cipher_key()[..]),
        send_key
    );
    assert_eq!(
        hex::encode(&responder_machine.chaining_key()[..]),
        chain_key
    );

    // Now test as per section "transport-message test" in Test Vectors
    // (the transportMessageVectors ciphertexts are from this section of BOLT 8);
    // we do slightly greater than 1000 encryption/decryption operations
    // to ensure that the key rotation algorithm is operating as expected.
    // The starting point for enc/decr is already guaranteed correct from the
    // above tests of sendingKey, receivingKey, chainingKey.
    let mut transport_message_vectors = HashMap::new();
    transport_message_vectors.insert(
        0,
        String::from(
            "cf2b30ddf0cf3f80e7c35a6e6730b59fe802473180f396d88a8fb0db8cbcf25d2f214cf9ea1d95",
        ),
    );
    transport_message_vectors.insert(
        1,
        String::from(
            "72887022101f0b6753e0c7de21657d35a4cb2a1f5cde2650528bbc8f837d0f0d7ad833b1a256a1",
        ),
    );
    transport_message_vectors.insert(
        500,
        String::from(
            "178cb9d7387190fa34db9c2d50027d21793c9bc2d40b1e14dcf30ebeeeb220f48364f7a4c68bf8",
        ),
    );
    transport_message_vectors.insert(
        501,
        String::from(
            "1b186c57d44eb6de4c057c49940d79bb838a145cb528d6e8fd26dbe50a60ca2c104b56b60e45bd",
        ),
    );
    transport_message_vectors.insert(
        1000,
        String::from(
            "4a2f3cc3b5e78ddb83dcb426d9863d9d9a723b0337c89dd0b005d89f8d3c05c52b76b29b740f09",
        ),
    );
    transport_message_vectors.insert(
        1001,
        String::from(
            "2ecd8c8a5629d0d02ab457a0fdd0f7b90a192cd46be5ecb6ca570bfc5e268338b1a16cf4ef2d36",
        ),
    );

    let payload = ('h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8);
    for i in 0..1002 {
        use bytes::BytesMut;

        let mut buffer = BytesMut::with_capacity(0x100);
        machine.write(payload.clone(), &mut buffer)?;

        if transport_message_vectors.get(&i).is_some() {
            let actual = hex::encode(buffer.as_mut());
            let expected = transport_message_vectors.get(&i).unwrap();
            assert_eq!(&actual, expected);
        }

        // Responder decrypts the bytes, in every iteration, and
        // should always be able to decrypt the same payload message.
        let plaintext = responder_machine.read(&mut buffer)?;
        // Ensure decryption succeeded
        assert_eq!(plaintext, Some(payload));
    }

    Ok(())
}
