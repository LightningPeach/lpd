use secp256k1::PublicKey;

// KEY_DERIVATION_VERSION is the version of the key derivation schema
// defined below. We use a version as this means that we'll be able to
// accept new seed in the future and be able to discern if the software
// is compatible with the version of the seed.
const KEY_DERIVATION_VERSION: u8 = 0;

// BIP0043_PURPOSE is the "purpose" value that we'll use for the first
// version or our key derivation scheme. All keys are expected to be
// derived from this purpose, then the particular coin type of the
// chain where the keys are to be used.  Slightly adhering to BIP0043
// allows us to not deviate too far from a widely used standard, and
// also fits into existing implementations of the BIP's template.
//
// NOTE: BRICK SQUUUUUAD.
pub const BIP0043_PURPOSE: u32 = 1017;

// KeyFamily represents a "family" of keys that will be used within various
// contracts created by lnd. These families are meant to be distinct branches
// within the HD key chain of the backing wallet. Usage of key families within
// the interface below are strict in order to promote integrability and the
// ability to restore all keys given a user master seed backup.
//
// The key derivation in this file follows the following hierarchy based on
// BIP43:
//
//   * m/1017'/coinType'/keyFamily/0/index
pub struct KeyFamily(pub u32);

// KEY_FAMILY_MULTI_SIG are keys to be used within multi-sig scripts.
const KEY_FAMILY_MULTI_SIG: KeyFamily = KeyFamily(0);

// KEY_FAMILY_REVOCATION_BASE are keys that are used within channels to
// create revocation basepoints that the remote party will use to
// create revocation keys for us.
const KEY_FAMILY_REVOCATION_BASE: KeyFamily = KeyFamily(1);

// KEY_FAMILY_HTLC_BASE are keys used within channels that will be
// combined with per-state randomness to produce public keys that will
// be used in HTLC scripts.
const KEY_FAMILY_HTLC_BASE: KeyFamily = KeyFamily(2);

// KEY_FAMILY_PAYMENT_BASE are keys used within channels that will be
// combined with per-state randomness to produce public keys that will
// be used in scripts that pay directly to us without any delay.
const KEY_FAMILY_PAYMENT_BASE: KeyFamily = KeyFamily(3);

// KEY_FAMILY_DELAY_BASE are keys used within channels that will be
// combined with per-state randomness to produce public keys that will
// be used in scripts that pay to us, but require a CSV delay before we
// can sweep the funds.
const KEY_FAMILY_DELAY_BASE: KeyFamily = KeyFamily(4);

// KEY_FAMILY_REVOCATION_ROOT is a family of keys which will be used to
// derive the root of a revocation tree for a particular channel.
const KEY_FAMILY_REVOCATION_ROOT: KeyFamily = KeyFamily(5);

// KEY_FAMILY_NODE_KEY is a family of keys that will be used to derive
// keys that will be advertised on the network to represent our current
// "identity" within the network. Peers will need our latest node key
// in order to establish a transport session with us on the Lightning
// p2p level (BOLT-0008).
const KEY_FAMILY_NODE_KEY: KeyFamily = KeyFamily(6);

// KeyLocator is a two-tuple that can be used to derive *any* key that has ever
// been used under the key derivation mechanisms described in this file.
// Version 0 of our key derivation schema uses the following BIP43-like
// derivation:
//
//   * m/201'/coinType'/keyFamily/0/index
//
// Our purpose is 201 (chosen arbitrary for now), and the coin type will vary
// based on which coin/chain the channels are being created on. The key family
// are actually just individual "accounts" in the nomenclature of BIP43. By
// default we assume a branch of 0 (external). Finally, the key index (which
// will vary per channel and use case) is the final element which allows us to
// deterministically derive keys.
pub struct KeyLocator {
	// TODO(roasbeef); add the key scope as well??

	// family is the family of key being identified.
	pub family: KeyFamily,

	// index is the precise index of the key being identified.
	pub index: u32,
}

// KeyDescriptor wraps a KeyLocator and also optionally includes a public key.
// Either the KeyLocator must be non-empty, or the public key pointer be
// non-nil. This will be used by the KeyRing interface to lookup arbitrary
// private keys, and also within the SignDescriptor struct to locate precisely
// which keys should be used for signing.
pub struct KeyDescriptor {
	// key_locator is the internal KeyLocator of the descriptor.
	pub key_locator: Option<KeyLocator>,

	// pub_key is an optional public key that fully describes a target key.
	// If this is nil, the KeyLocator MUST NOT be empty.
	pub pub_key: Option<PublicKey>,
}