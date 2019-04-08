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

func main() {
	var f, err = os.Create("../target/messages")
	if err != nil {
		panic(err)
	}
	defer f.Close()

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
	writeMessage(f, &lnwire.FundingLocked{})
}
