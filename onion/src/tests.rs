use super::{OnionPacketVersion, OnionPacket, ValidOnionPacket, OnionRoute, Hop, HopData, BitcoinHopData, Processed};
use secp256k1::SecretKey;
use wire::{Wrapper, Satoshi};
use binformat::BinarySD;

// BOLT4_PUB_KEYS are the public keys of the hops used in the route.
const BOLT4_PUB_KEYS: [&str; 5] = [
    "02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619",
    "0324653eac434488002cc06bbfb7f10fe18991e35f9fe4302dbea6d2353dc0ab1c",
    "027f31ebc5462c1fdce1b737ecff52d37d75dea43ce11c74d25aa297165faa2007",
    "032c0b7cf95324a07d05398b240174dc0c2be444d96b159aa6c7f7b1e668680991",
    "02edabbd16b41c8371b92ef2f04c1185b4f03b6dcd52ba9b78d9d7c89c8f221145",
];

// BOLT4_SESSION_KEY is the session private key.
const BOLT4_SESSION_KEY: &'static str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

// BOLT4_ASSOC_DATA is the associated data added to the packet.
const BOLT4_ASSOC_DATA: &'static str = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";

// bolt4_final_packet_hex encodes the expected sphinx packet as a result of
// creating a new packet with the above parameters.
const BOLT4_FINAL_PACKET_HEX: &'static str = "\
                                              0002eec7245d6b7d2ccb30380bfbe2a3648cd7\
                                              a942653f5aa340edcea1f283686619e5f14350c2a76fc232b5e4\
                                              6d421e9615471ab9e0bc887beff8c95fdb878f7b3a71da571226\
                                              458c510bbadd1276f045c21c520a07d35da256ef75b436796243\
                                              7b0dd10f7d61ab590531cf08000178a333a347f8b4072e216400\
                                              406bdf3bf038659793a86cae5f52d32f3438527b47a1cfc54285\
                                              a8afec3a4c9f3323db0c946f5d4cb2ce721caad69320c3a469a2\
                                              02f3e468c67eaf7a7cda226d0fd32f7b48084dca885d15222e60\
                                              826d5d971f64172d98e0760154400958f00e86697aa1aa9d41be\
                                              e8119a1ec866abe044a9ad635778ba61fc0776dc832b39451bd5\
                                              d35072d2269cf9b040d6ba38b54ec35f81d7fc67678c3be47274\
                                              f3c4cc472aff005c3469eb3bc140769ed4c7f0218ff8c6c7dd72\
                                              21d189c65b3b9aaa71a01484b122846c7c7b57e02e679ea8469b\
                                              70e14fe4f70fee4d87b910cf144be6fe48eef24da475c0b0bcc6\
                                              565ae82cd3f4e3b24c76eaa5616c6111343306ab35c1fe5ca4a7\
                                              7c0e314ed7dba39d6f1e0de791719c241a939cc493bea2bae1c1\
                                              e932679ea94d29084278513c77b899cc98059d06a27d171b0dbd\
                                              f6bee13ddc4fc17a0c4d2827d488436b57baa167544138ca2e64\
                                              a11b43ac8a06cd0c2fba2d4d900ed2d9205305e2d7383cc98dac\
                                              b078133de5f6fb6bed2ef26ba92cea28aafc3b9948dd9ae5559e\
                                              8bd6920b8cea462aa445ca6a95e0e7ba52961b181c79e73bd581\
                                              821df2b10173727a810c92b83b5ba4a0403eb710d2ca10689a35\
                                              bec6c3a708e9e92f7d78ff3c5d9989574b00c6736f84c199256e\
                                              76e19e78f0c98a9d580b4a658c84fc8f2096c2fbea8f5f8c59d0\
                                              fdacb3be2802ef802abbecb3aba4acaac69a0e965abd8981e989\
                                              6b1f6ef9d60f7a164b371af869fd0e48073742825e9434fc54da\
                                              837e120266d53302954843538ea7c6c3dbfb4ff3b2fdbe244437\
                                              f2a153ccf7bdb4c92aa08102d4f3cff2ae5ef86fab4653595e6a\
                                              5837fa2f3e29f27a9cde5966843fb847a4a61f1e76c281fe8bb2\
                                              b0a181d096100db5a1a5ce7a910238251a43ca556712eaadea16\
                                              7fb4d7d75825e440f3ecd782036d7574df8bceacb397abefc5f5\
                                              254d2722215c53ff54af8299aaaad642c6d72a14d27882d9bbd5\
                                              39e1cc7a527526ba89b8c037ad09120e98ab042d3e8652b31ae0\
                                              e478516bfaf88efca9f3676ffe99d2819dcaeb7610a626695f53\
                                              117665d267d3f7abebd6bbd6733f645c72c389f03855bdf1e4b8\
                                              075b516569b118233a0f0971d24b83113c0b096f5216a207ca99\
                                              a7cddc81c130923fe3d91e7508c9ac5f2e914ff5dccab9e55856\
                                              6fa14efb34ac98d878580814b94b73acbfde9072f30b881f7f0f\
                                              ff42d4045d1ace6322d86a97d164aa84d93a60498065cc7c20e6\
                                              36f5862dc81531a88c60305a2e59a985be327a6902e4bed986db\
                                              f4a0b50c217af0ea7fdf9ab37f9ea1a1aaa72f54cf40154ea9b2\
                                              69f1a7c09f9f43245109431a175d50e2db0132337baa0ef97eed\
                                              0fcf20489da36b79a1172faccc2f7ded7c60e00694282d93359c\
                                              4682135642bc81f433574aa8ef0c97b4ade7ca372c5ffc23c7ed\
                                              dd839bab4e0f14d6df15c9dbeab176bec8b5701cf054eb3072f6\
                                              dadc98f88819042bf10c407516ee58bce33fbe3b3d86a54255e5\
                                              77db4598e30a135361528c101683a5fcde7e8ba53f3456254be8\
                                              f45fe3a56120ae96ea3773631fcb3873aa3abd91bcff00bd38bd\
                                              43697a2e789e00da6077482e7b1b1a677b5afae4c54e6cbdf737\
                                              7b694eb7d7a5b913476a5be923322d3de06060fd5e819635232a\
                                              2cf4f0731da13b8546d1d6d4f8d75b9fce6c2341a71b0ea6f780\
                                              df54bfdb0dd5cd9855179f602f917265f21f9190c70217774a6f\
                                              baaa7d63ad64199f4664813b955cff954949076dcf";

#[test]
fn test_bolt4_packet() {
    use secp256k1::PublicKey;

    let path = BOLT4_PUB_KEYS
        .iter()
        .enumerate()
        .map(|(i, &d)| {
            let pk_bytes = hex::decode(d)
                .expect(format!("unable to decode BOLT 4 hex pubkey #{}", i).as_str());
            let pk = PublicKey::from_slice(&pk_bytes)
                .expect(format!("unable to parse BOLT 4 pubkey #{}", i).as_str());

            let next_address = BinarySD::deserialize(&[i as u8; 8][..]).unwrap();

            let hop_data = HopData::Bitcoin(BitcoinHopData::new(
                next_address,
                Satoshi::default().fmap(|_| i as _),
                i as u32,
            ));
            Hop::new(pk, hop_data)
        }).collect::<Vec<_>>();

    let route = OnionRoute::new(
        OnionPacketVersion::_0,
        SecretKey::from_slice(BOLT4_SESSION_KEY.as_bytes()).unwrap(),
        path,
        BOLT4_ASSOC_DATA.as_bytes().to_vec(),
    );

    let packet = route.packet().unwrap();
    let reference_packet_bytes = hex::decode(BOLT4_FINAL_PACKET_HEX.to_owned())
        .expect("unable to decode BOLT 4 final onion packet from hex");
    let reference_packet: OnionPacket =
        BinarySD::deserialize(reference_packet_bytes.as_slice()).unwrap();
    let valid_reference_packet = reference_packet.validate().unwrap();
    assert_eq!(packet, valid_reference_packet);
}

fn new_test_route(num_hops: usize) -> (Vec<SecretKey>, Vec<Hop>, ValidOnionPacket) {
    use secp256k1::{Secp256k1, PublicKey};

    let context = Secp256k1::new();

    let mut rng = rand::thread_rng();
    let keys = (0..num_hops)
        .map(|_| SecretKey::new(&mut rng))
        .collect::<Vec<SecretKey>>();

    let hops = keys
        .iter()
        .enumerate()
        .map(|(i, secret_key)| {
            let data = HopData::Bitcoin(BitcoinHopData::new(
                BinarySD::deserialize(&[i as u8; 8][..]).unwrap(),
                Satoshi::default().fmap(|_| i as _),
                i as u32,
            ));
            Hop::new(
                PublicKey::from_secret_key(&context, secret_key),
                data,
            )
        }).collect::<Vec<_>>();

    // Generate a forwarding message to route to the final node via the
    // generated intermediates nodes above. Destination should be Hash160,
    // adding padding so parsing still works.
    let route = OnionRoute::new(
        OnionPacketVersion::_0,
        SecretKey::from_slice(BOLT4_SESSION_KEY.as_bytes()).unwrap(),
        hops.clone(),
        vec![],
    );

    (keys, hops, route.packet().unwrap())
}

#[test]
fn test_sphinx_correctness() {
    let (keys, hops, packet) = new_test_route(OnionPacket::NUM_MAX_HOPS);

    // Now simulate the message propagating through the mix net eventually
    // reaching the final destination.
    let none = keys.iter().zip(hops).enumerate().fold(
        Some(packet),
        |maybe_packet, (i, (secret_key, expected_hop))| {
            let packet = maybe_packet.unwrap();

            println!("processing at hop: {}", i);
            let processed = packet
                .process(vec![], (i as u32) + 1, secret_key)
                .unwrap();

            match processed {
                Processed::ExitNode => None,
                Processed::MoreHops {
                    next: packet,
                    forwarding_instructions: hop,
                } => {
                    assert_eq!(hop, expected_hop.data());
                    Some(packet.validate().unwrap())
                }
            }
        },
    );

    assert_eq!(none, None);
}

#[test]
fn test_sphinx_single_hop() {
    // We'd like to test the proper behavior of the correctness of onion
    // packet processing for "single-hop" payments which bare a full onion
    // packet.
    let (keys, _, fwd_msg) = new_test_route(1);

    // Simulating a direct single-hop payment, send the sphinx packet to
    // the destination node, making it process the packet fully.
    let processed = fwd_msg
        .process(vec![], 1, &keys[0])
        .expect("unable to process sphinx packet");

    // The destination node should detect that the packet is destined for
    // itself.
    assert_eq!(processed, Processed::ExitNode);
}

#[test]
#[should_panic]
fn test_sphinx_assoc_data() {
    // We want to make sure that the associated data is considered in the
    // HMAC creation
    let (keys, _, fwd_msg) = new_test_route(5);

    let processed = fwd_msg
        .process("something else".as_bytes().to_vec(), 1, &keys[0])
        .unwrap();
    match processed {
        Processed::ExitNode => (),
        Processed::MoreHops {
            next: next,
            forwarding_instructions: _,
        } => {
            let _ = next.validate().unwrap();
        }
    };
}

#[test]
fn test_sphinx_serde() {
    // Create some test data with a randomly populated, yet valid onion
    // forwarding message.
    let (_, _, fwd_msg) = new_test_route(5);

    // Encode the created onion packet into an empty buffer. This should
    // succeed without any errors.
    let mut buffer = Vec::new();
    BinarySD::serialize(&mut buffer, &fwd_msg).expect("unable to serialize message");

    // Now decode the bytes encoded above. Again, this should succeed
    // without any errors.
    let new_fwd_msg: OnionPacket =
        BinarySD::deserialize(buffer.as_slice()).expect("unable to deserialize message");
    let new_fwd_msg = new_fwd_msg
        .validate()
        .expect("deserialized message is invalid");

    // The two forwarding messages should now be identical.
    assert_eq!(
        fwd_msg, new_fwd_msg,
        "forwarding messages don't match, {:?} vs {:?}",
        fwd_msg, new_fwd_msg
    );
}
