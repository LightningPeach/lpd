package main

import (
	"bytes"
	"encoding/binary"
	"io"
	"os"

	"github.com/btcsuite/btcd/btcec"
	"github.com/lightningnetwork/lnd/lnwire"
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

	writeMessage(f, &lnwire.OpenChannel{
		FundingKey:           pubkey,
		RevocationPoint:      pubkey,
		PaymentPoint:         pubkey,
		DelayedPaymentPoint:  pubkey,
		HtlcPoint:            pubkey,
		FirstCommitmentPoint: pubkey,
	})
	writeMessage(f, &lnwire.AcceptChannel{
		FundingKey:           pubkey,
		RevocationPoint:      pubkey,
		PaymentPoint:         pubkey,
		DelayedPaymentPoint:  pubkey,
		HtlcPoint:            pubkey,
		FirstCommitmentPoint: pubkey,
	})
	writeMessage(f, &lnwire.FundingCreated{})
	writeMessage(f, &lnwire.FundingSigned{})
	writeMessage(f, createRevokeAndAck())
	writeMessage(f, createFundingLocked())
	writeMessage(f, createAnnounceSignatures())
}
