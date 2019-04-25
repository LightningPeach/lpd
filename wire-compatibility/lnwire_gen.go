package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/binary"
	"github.com/btcsuite/btcutil"
	"image/color"
	"io"
	"net"
	"os"

	"github.com/btcsuite/btcd/btcec"
	"github.com/btcsuite/btcd/chaincfg/chainhash"
	"github.com/lightningnetwork/lnd/lnwire"

	"github.com/btcsuite/btcd/wire"
)

func randPubKey() (*btcec.PublicKey, error) {
	priv, err := btcec.NewPrivateKey(btcec.S256())
	if err != nil {
		return nil, err
	}

	return priv.PubKey(), nil
}

func writeMessage(w io.Writer, msg lnwire.Message) {
	var b = bytes.NewBuffer(make([]byte, 0))
	var _, err = lnwire.WriteMessage(b, msg, 0)
	if err != nil {
		panic(err)
	}
	err = binary.Write(w, binary.BigEndian, uint16(b.Len()))
	if err != nil {
		panic(err)
	}
	_, err = w.Write(b.Bytes())
	if err != nil {
		panic(err)
	}
}


func createRevokeAndAck() lnwire.Message {
	var chanId lnwire.ChannelID
	var revocation [32]byte
	chanId[0] = 1
	revocation[1] = 2
	privkey, _ := btcec.NewPrivateKey(btcec.S256())
	pubkey := privkey.PubKey()
	msg := &lnwire.RevokeAndAck{
		ChanID:            chanId,
		Revocation:        revocation,
		NextRevocationKey: pubkey,
	}
	return msg
}

func createFundingLocked() lnwire.Message {
	var chanId lnwire.ChannelID
	chanId[0] = 1
	privkey, _ := btcec.NewPrivateKey(btcec.S256())
	pubkey := privkey.PubKey()
	msg := &lnwire.FundingLocked{
		ChanID: chanId,
		NextPerCommitmentPoint: pubkey,
	}
	return msg
}

func createAnnounceSignatures() lnwire.Message {
	var chanId lnwire.ChannelID
	chanId[0] = 1
	shortChannelID := lnwire.ShortChannelID{
		BlockHeight: 100,
		TxIndex: 10,
		TxPosition: 1,
	}
	var nodeSignature lnwire.Sig
	nodeSignature[2] = 3

	var bitcoinSignature lnwire.Sig
	bitcoinSignature[1] = 4

	extraOpaqueData := []byte{1, 2, 3, 100}

	msg := &lnwire.AnnounceSignatures {
		ChannelID: chanId,
		ShortChannelID: shortChannelID,
		NodeSignature: nodeSignature,
		BitcoinSignature: bitcoinSignature,
		ExtraOpaqueData: extraOpaqueData,
	}
	return msg
}

func as33Byte(a []byte) [33]byte {
	if len(a) != 33{
		panic("incorrect length")
	}
	var b [33]byte
	copy(b[:], a)

	return b
}

func createChannelAnnouncement() lnwire.Message {
	var nodeSig1 lnwire.Sig
	nodeSig1[2] = 3

	var nodeSig2 lnwire.Sig
	nodeSig2[3] = 4

	var bitcoinSig1 lnwire.Sig
	bitcoinSig1[1] = 2

	var bitcoinSig2 lnwire.Sig
	bitcoinSig2[2] = 5

	features := lnwire.NewRawFeatureVector(lnwire.DataLossProtectOptional, lnwire.GossipQueriesOptional)

	var chainHash chainhash.Hash
	chainHash[1] = 11

	shortChannelId := lnwire.ShortChannelID{
		TxPosition: 100,
		TxIndex: 2,
		BlockHeight: 1234,
	}

	nodeId1, _ := randPubKey()
	nodeId2,_  := randPubKey()

	bitcoinKey1, _ := randPubKey()

	bitcoinKey2, _ := randPubKey()

	// TODO(mkl): do not exist in spec, but do exist in lnd
	extraOpaqueData := []byte{}

	msg := &lnwire.ChannelAnnouncement{
		NodeSig1: nodeSig1,
		NodeSig2: nodeSig2,

		BitcoinSig1: bitcoinSig1,
		BitcoinSig2: bitcoinSig2,

		Features: features,

		ChainHash:chainHash,

		ShortChannelID: shortChannelId,

		NodeID1: as33Byte(nodeId1.SerializeCompressed()),
		NodeID2: as33Byte(nodeId2.SerializeCompressed()),

		BitcoinKey1: as33Byte(bitcoinKey1.SerializeCompressed()),
		BitcoinKey2: as33Byte(bitcoinKey2.SerializeCompressed()),

		ExtraOpaqueData: extraOpaqueData,
	}

	return msg
}

func createChannelReestablish() lnwire.Message {
	var chanId lnwire.ChannelID
	chanId[0] = 1

	nextLocalCommitHeight := uint64(11)

	remoteCommitTailHeight := uint64(2)

	var lastRemoteCommitSecret [32]byte
	lastRemoteCommitSecret[1] = 2

	localUnrevokedCommitPoint, _ := randPubKey()

	msg := &lnwire.ChannelReestablish{
		ChanID: chanId,
		NextLocalCommitHeight: nextLocalCommitHeight,
		RemoteCommitTailHeight: remoteCommitTailHeight,
		LastRemoteCommitSecret: lastRemoteCommitSecret,
		LocalUnrevokedCommitPoint: localUnrevokedCommitPoint,
	}
	return msg
}

func createChannelUpdate() lnwire.Message {
	var signature lnwire.Sig
	signature[2] = 3

	var chainHash chainhash.Hash
	chainHash[1] = 4

	shortChannelId := lnwire.ShortChannelID{
		BlockHeight: 100,
		TxIndex: 4,
		TxPosition: 15,
	}

	timestamp := uint32(1000000)

	messageFlags := lnwire.ChanUpdateOptionMaxHtlc

	channelFlags := lnwire.ChanUpdateDirection

	timeLockDelta := uint16(100)

	htlcMinimumMsat := lnwire.MilliSatoshi(1000)

	baseFee := uint32(100)

	feeRate := uint32(5)

	htlcMaximumMsat := lnwire.MilliSatoshi(100000000)

	extraOpaqueData := []byte{}

	msg := &lnwire.ChannelUpdate {
		Signature: signature,
		ChainHash: chainHash,
		ShortChannelID: shortChannelId,
		Timestamp: timestamp,
		MessageFlags: messageFlags,
		ChannelFlags: channelFlags,
		TimeLockDelta: timeLockDelta,
		HtlcMinimumMsat: htlcMinimumMsat,
		BaseFee: baseFee,
		FeeRate: feeRate,
		HtlcMaximumMsat: htlcMaximumMsat,
		ExtraOpaqueData: extraOpaqueData,
	}

	return msg

}

func createClosingSigned() lnwire.Message {
	var chanId lnwire.ChannelID
	chanId[0] = 1

	feeSatoshis := btcutil.Amount(123)

	var signature lnwire.Sig
	signature[1] = 2

	msg := &lnwire.ClosingSigned{
		ChannelID: chanId,
		FeeSatoshis: feeSatoshis,
		Signature: signature,
	}

	return msg

}

func createCommitSig() lnwire.Message {
	var chanId lnwire.ChannelID
	chanId[0] = 1

	var commitSig lnwire.Sig
	commitSig[1] = 2

	var htlcSig1 lnwire.Sig
	htlcSig1[1] = 3

	var htlcSig2 lnwire.Sig
	htlcSig2[3] = 4

	htlcSigs := []lnwire.Sig{htlcSig1, htlcSig2}

	msg := &lnwire.CommitSig{
		ChanID: chanId,
		CommitSig: commitSig,
		HtlcSigs: htlcSigs,
	}

	return msg
}

func createFundingCreated() lnwire.Message {
	var pendingChannelID [32]byte
	pendingChannelID[0] = 2

	var fundingPointHash chainhash.Hash
	fundingPointHash[3] = 5

	fundingPoint := wire.OutPoint{
		Index: 2,
		Hash: fundingPointHash,
	}

	var commitSig lnwire.Sig
	commitSig[1] = 5

	msg := &lnwire.FundingCreated{
		PendingChannelID: pendingChannelID,
		FundingPoint: fundingPoint,
		CommitSig: commitSig,
	}

	return msg
}

func createFundingSigned() lnwire.Message {
	var chanId lnwire.ChannelID
	chanId[1] = 5

	var commitSig lnwire.Sig
	commitSig[1] = 3

	msg := &lnwire.FundingSigned{
		ChanID: chanId,
		CommitSig: commitSig,
	}
	return msg
}

func createGossipTimestampRange() lnwire.Message {
	var chainHash chainhash.Hash
	chainHash[1] = 11

	firstTimestamp := uint32(100000000)

	timestampRange := uint32(1234)

	msg := &lnwire.GossipTimestampRange{
		ChainHash: chainHash,
		FirstTimestamp: firstTimestamp,
		TimestampRange: timestampRange,
	}

	return msg
}

func createInit() lnwire.Message {
	globalFeatures := lnwire.NewRawFeatureVector()

	localFeatures := lnwire.NewRawFeatureVector(
		lnwire.DataLossProtectRequired,
		lnwire.DataLossProtectOptional,
		lnwire.InitialRoutingSync,
		lnwire.GossipQueriesRequired,
		lnwire.GossipQueriesOptional,
	)

	msg := &lnwire.Init{
		GlobalFeatures: globalFeatures,
		LocalFeatures: localFeatures,
	}

	return msg

}

func createNodeAnnouncement() lnwire.Message {
	var signature lnwire.Sig
	signature[3] = 1

	features := lnwire.NewRawFeatureVector()

	timestamp := uint32(123331122)

	pubKey, _ := randPubKey()
	nodeId := as33Byte(pubKey.SerializeCompressed())

	rgbColor := color.RGBA{
		A: 100,
		B: 10,
		G: 20,
		R: 33,
	}

	var alias lnwire.NodeAlias
	alias[0] = 1

	addresses := []net.Addr{
		&net.TCPAddr{
				IP: net.ParseIP("127.0.0.1"),
				Port: 10000,
				Zone: "",
		},
		&net.TCPAddr{
			IP: net.ParseIP("9.9.9.9"),
			Port: 1234,
			Zone: "",
		},
		&net.TCPAddr{
			IP: net.ParseIP("10.10.101.21"),
			Port: 11111,
			Zone: "",
		},

	}

	extraOpaqueData := []byte{}

	msg := &lnwire.NodeAnnouncement{
		Signature: signature,
		Features: features,
		Timestamp: timestamp,
		NodeID: nodeId,
		RGBColor: rgbColor,
		Alias: alias,
		Addresses: addresses,
		ExtraOpaqueData: extraOpaqueData,
	}

	return msg
}

func createOpenChannel() lnwire.Message {
	var chainHash chainhash.Hash
	chainHash[2] = 12

	var pendingChannelId [32]byte
	pendingChannelId[0] = 2

	fundingAmount := btcutil.Amount(100000)

	pushAmount := lnwire.MilliSatoshi(12341)

	dustLimit := btcutil.Amount(200)

	maxValueInFlight := lnwire.MilliSatoshi(10000)

	channelReserve := btcutil.Amount(1000)

	htlcMinimum := lnwire.MilliSatoshi(1000)

	feePerKiloWeight := uint32(10)

	csvDelay := uint16(15)

	maxAcceptedHTLCs := uint16(10)

	fundingKey, _ := randPubKey()

	revocationPoint, _ := randPubKey()

	paymentPoint, _ := randPubKey()

	delayedPaymentPoint, _ := randPubKey()

	htlcPoint, _ := randPubKey()

	firstCommitmentPoint, _ := randPubKey()

	channelFlags := lnwire.FFAnnounceChannel

	msg := &lnwire.OpenChannel{
		ChainHash: chainHash,
		PendingChannelID: pendingChannelId,
		FundingAmount: fundingAmount,
		PushAmount: pushAmount,
		DustLimit: dustLimit,
		MaxValueInFlight: maxValueInFlight,
		ChannelReserve: channelReserve,
		HtlcMinimum: htlcMinimum,
		FeePerKiloWeight: feePerKiloWeight,
		CsvDelay: csvDelay,
		MaxAcceptedHTLCs: maxAcceptedHTLCs,
		FundingKey: fundingKey,
		RevocationPoint: revocationPoint,
		PaymentPoint: paymentPoint,
		DelayedPaymentPoint: delayedPaymentPoint,
		HtlcPoint: htlcPoint,
		FirstCommitmentPoint: firstCommitmentPoint,
		ChannelFlags: channelFlags,
	}

	return msg
}

func createPing() lnwire.Message {
	numPongBytes := uint16(10)
	paddingBytes := lnwire.PingPayload{1, 2, 3, 4}
	msg := &lnwire.Ping{
		NumPongBytes: numPongBytes,
		PaddingBytes: paddingBytes,
	}
	return msg
}

func createPong() lnwire.Message {
	pongBytes := lnwire.PongPayload{1, 200}
	msg := &lnwire.Pong{
		PongBytes: pongBytes,
	}
	return msg
}

func createQueryChannelRange() lnwire.Message {
	var chainHash chainhash.Hash
	chainHash[2] = 11

	firstBlockHeight := uint32(10000)

	numBlocks := uint32(12)

	msg := &lnwire.QueryChannelRange{
		ChainHash: chainHash,
		FirstBlockHeight: firstBlockHeight,
		NumBlocks: numBlocks,
	}

	return msg
}

func createQueryShortChanIDs() lnwire.Message {
	var chainHash chainhash.Hash
	chainHash[1] = 6

	encodingType := lnwire.EncodingSortedPlain

	shortChanIdS := []lnwire.ShortChannelID{
		lnwire.ShortChannelID{
			TxPosition: 10,
			TxIndex: 1,
			BlockHeight: 12221,
		},
		lnwire.ShortChannelID{
			TxPosition: 101,
			TxIndex: 12,
			BlockHeight: 15000,
		},
		lnwire.ShortChannelID{
			TxPosition: 11,
			TxIndex: 5,
			BlockHeight: 200000,
		},
	}
	msg := &lnwire.QueryShortChanIDs{
		ChainHash: chainHash,
		EncodingType: encodingType,
		ShortChanIDs: shortChanIdS,
	}

	return msg
}

func createReplyChannelRange() lnwire.Message {
	queryChannelRange := (createQueryChannelRange()).(*lnwire.QueryChannelRange)

	complete := uint8(1)

	encodingType := lnwire.EncodingSortedPlain

	shortChanIdS := []lnwire.ShortChannelID{
		lnwire.ShortChannelID{
			TxPosition: 10,
			TxIndex: 1,
			BlockHeight: 12221,
		},
		lnwire.ShortChannelID{
			TxPosition: 101,
			TxIndex: 12,
			BlockHeight: 15000,
		},
		lnwire.ShortChannelID{
			TxPosition: 11,
			TxIndex: 5,
			BlockHeight: 200000,
		},
	}

	msg := &lnwire.ReplyChannelRange{
		QueryChannelRange: *queryChannelRange,
		Complete: complete,
		EncodingType: encodingType,
		ShortChanIDs: shortChanIdS,
	}

	return msg
}

func createReplyShortChanIDsEnd() lnwire.Message {
	var chainHash chainhash.Hash
	chainHash[1] = 7

	complete := uint8(1)

	msg := &lnwire.ReplyShortChanIDsEnd {
		ChainHash: chainHash,
		Complete: complete,
	}

	return msg
}

func createShutdown() lnwire.Message {
	var chanelId lnwire.ChannelID
	chanelId[0] = 1

	address := lnwire.DeliveryAddress{1, 2, 3, 4, 5}

	msg := &lnwire.Shutdown{
		ChannelID: chanelId,
		Address: address,
	}

	return msg
}

func createUpdateAddHTLC() lnwire.Message {
	var chanID lnwire.ChannelID
	chanID[0] = 2

	iD := uint64(1001)

	amount := lnwire.MilliSatoshi(101000)

	var paymentHash [32]byte
	paymentHash[1] = 121

	expiry := uint32(100)

	var onionBlob [lnwire.OnionPacketSize]byte
	onionBlob[1] = 5

	msg := &lnwire.UpdateAddHTLC{
		ChanID: chanID,
		ID: iD,
		Amount: amount,
		PaymentHash: paymentHash,
		Expiry: expiry,
		OnionBlob: onionBlob,
	}
	return msg
}

func createUpdateFailHTLC() lnwire.Message {
	var chanID lnwire.ChannelID
	chanID[0] = 4

	iD := uint64(10010)

	reason := lnwire.OpaqueReason{1, 2, 1, 5}

	msg := &lnwire.UpdateFailHTLC{
		ChanID: chanID,
		ID: iD,
		Reason: reason,
	}

	return msg
}

func createUpdateFailMalformedHTLC() lnwire.Message {
	var chanID lnwire.ChannelID
	chanID[0] = 2

	iD := uint64(100)

	var shaOnionBlob [sha256.Size]byte
	shaOnionBlob[3] = 5

	failureCode := lnwire.CodeInvalidOnionHmac

	msg := &lnwire.UpdateFailMalformedHTLC{
		ChanID: chanID,
		ID: iD,
		ShaOnionBlob: shaOnionBlob,
		FailureCode: failureCode,
	}

	return msg
}

func createUpdateFee() lnwire.Message {
	var chanID lnwire.ChannelID
	chanID[0] = 2

	feePerKw := uint32(1001)

	msg := &lnwire.UpdateFee{
		ChanID: chanID,
		FeePerKw: feePerKw,
	}

	return msg
}

func createUpdateFulfillHtlc() lnwire.Message {
	var chanID lnwire.ChannelID
	chanID[0] = 2

	iD := uint64(121)

	var paymentPreimage [32]byte
	paymentPreimage[1] = 100

	msg := &lnwire.UpdateFulfillHTLC{
		ChanID: chanID,
		ID: iD,
		PaymentPreimage: paymentPreimage,
	}

	return msg
}


func main() {
	var f, err = os.Create("/tmp/messages")
	if err != nil {
		panic(err)
	}
	defer f.Close()

	// TODO(mkl): refactor into some object, like TestDataProducer
	pubkey, err := randPubKey()
	if err != nil {
		panic(err)
	}

	writeMessage(f, createOpenChannel())
	writeMessage(f, &lnwire.AcceptChannel{
		FundingKey:           pubkey,
		RevocationPoint:      pubkey,
		PaymentPoint:         pubkey,
		DelayedPaymentPoint:  pubkey,
		HtlcPoint:            pubkey,
		FirstCommitmentPoint: pubkey,
	})
	writeMessage(f, createFundingCreated())
	writeMessage(f, createFundingSigned())
	writeMessage(f, createRevokeAndAck())
	writeMessage(f, createFundingLocked())

	// TODO(mkl): wtf???
	//writeMessage(f, createAnnounceSignatures())

	// TODO(mkl): issue with extra data
	// writeMessage(f, createChannelAnnouncement())

	writeMessage(f, createChannelReestablish())

	// TODO(mkl): wtf ???
	//writeMessage(f, createChannelUpdate())

	writeMessage(f, createClosingSigned())

	// TODO(mkl): wtf ???
	//writeMessage(f, createCommitSig())

	writeMessage(f, createGossipTimestampRange())

	writeMessage(f, createInit())

	// TODO(mkl): serialization gives one less address then needed
	writeMessage(f, createNodeAnnouncement())

	writeMessage(f, createPing())

	writeMessage(f, createPong())

	writeMessage(f, createQueryChannelRange())

	writeMessage(f, createQueryShortChanIDs())

	writeMessage(f, createReplyChannelRange())

	writeMessage(f, createReplyShortChanIDsEnd())

	writeMessage(f, createShutdown())

	writeMessage(f, createUpdateAddHTLC())

	writeMessage(f, createUpdateFailHTLC())

	writeMessage(f, createUpdateFailMalformedHTLC())

	writeMessage(f, createUpdateFee())

	writeMessage(f, createUpdateFulfillHtlc())
}

