use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey, ChildNumber};
use bitcoin::network::constants::Network;
use secp256k1::{Secp256k1, SecretKey};

use std::error::Error;

use super::derivation::{KeyLocator, KeyDescriptor, BIP0043_PURPOSE};

// COIN_TYPE_BITCOIN specifies the BIP44 coin type for Bitcoin key
// derivation.
const COIN_TYPE_BITCOIN: u32 = 0;

// COIN_TYPE_TESTNET specifies the BIP44 coin type for all testnet key
// derivation.
const COIN_TYPE_TESTNET: u32 = 1;

// COIN_TYPE_LITECOIN specifies the BIP44 coin type for Litecoin key
// derivation.
const COIN_TYPE_LITECOIN: u32 = 2;

struct KeyRing {
    key_manager: KeyManager,
    key_scope:   KeyScope,
}

impl KeyRing {
    fn new(seed: &[u8], coin_type: u32) -> Result<Self, Box<Error>> {
        let key_manager = KeyManager::from_seed(seed)?;
        let key_scope = KeyScope {
            purpose: BIP0043_PURPOSE,
            coin:    coin_type,
        };

        Ok(Self {
            key_manager,
            key_scope,
        })
    }

    // derive_key attempts to derive an arbitrary key specified by the passed
    // KeyLocator. This may be used in several recovery scenarios, or when manually
    // rotating something like our current default node key.
    //
    // NOTE: This is part of the keychain.KeyRing interface.
    fn derive_key(&self, key_loc: KeyLocator) -> Result<KeyDescriptor, Box<Error>> {
        let derivation_path = DerivationPath {
            account: key_loc.family.0,
            branch:  0,
            index:   key_loc.index,
        };

        let extended_pub_key = self.key_manager.derive_public_key_from_path(&self.key_scope, &derivation_path)?;

        Ok(KeyDescriptor{
            key_locator: Some(key_loc),
            pub_key:     Some(extended_pub_key.public_key),
        })
    }

    // derive_priv_key attempts to derive the private key that corresponds to the
    // passed key descriptor.
    //
    // NOTE: This is part of the keychain.SecretKeyRing interface.
    fn derive_priv_key(&self, key_desc: &KeyDescriptor) -> Result<SecretKey, Box<Error>> {
        let err: Box<Error> = From::from("key locator must be initialized");
        let key_loc = key_desc.key_locator.as_ref().ok_or(err)?;

        let derivation_path = DerivationPath {
            account: key_loc.family.0,
            branch:  0,
            index:   key_loc.index,
        };

        let extended_priv_key = self.key_manager.derive_private_key_from_path(&self.key_scope, &derivation_path)?;

        Ok(extended_priv_key.secret_key)
    }
}

struct KeyManager {
    master_key: ExtendedPrivKey,
}

impl KeyManager {
    fn from_seed(seed: &[u8]) -> Result<Self, Box<Error>> {
        Ok(Self {
            master_key: ExtendedPrivKey::new_master(&Secp256k1::new(), Network::Bitcoin, seed)?,
        })
    }

    // TODO(evg): use another key derivation scheme?
    fn derive_public_key_from_path(&self, key_scope: &KeyScope, derivation_path: &DerivationPath) -> Result<ExtendedPubKey, Box<Error>> {
        let extended_priv_key = self.derive_private_key_from_path(key_scope, derivation_path)?;
        Ok(ExtendedPubKey::from_private(&Secp256k1::new(), &extended_priv_key))
    }

    fn derive_private_key_from_path(&self, key_scope: &KeyScope, derivation_path: &DerivationPath) -> Result<ExtendedPrivKey, Box<Error>> {
        let path: &[ChildNumber] = &[
            ChildNumber::Hardened(key_scope.purpose),
            ChildNumber::Hardened(key_scope.coin),

            ChildNumber::Hardened(derivation_path.account),
            ChildNumber::Normal(derivation_path.branch),
            ChildNumber::Normal(derivation_path.index),
        ];

        let derived_key = ExtendedPrivKey::from_path(&Secp256k1::new(), &self.master_key, path)?;
        Ok(derived_key)
    }
}

// DerivationPath represents a derivation path from a particular key manager's
// scope.  Each ScopedKeyManager starts key derivation from the end of their
// cointype hardened key: m/purpose'/cointype'. The fields in this struct allow
// further derivation to the next three child levels after the coin type key.
// This restriction is in the spriti of BIP0044 type derivation. We maintain a
// degree of coherency with the standard, but allow arbitrary derivations
// beyond the cointype key. The key derived using this path will be exactly:
// m/purpose'/cointype'/account/branch/index, where purpose' and cointype' are
// bound by the scope of a particular manager.
struct DerivationPath {
	// account is the account, or the first immediate child from the scoped
	// manager's hardened coin type key.
	account: u32,

	// branch is the branch to be derived from the account index above. For
	// BIP0044-like derivation, this is either 0 (external) or 1
	// (internal). However, we allow this value to vary arbitrarily within
	// its size range.
	branch: u32,

	// index is the final child in the derivation path. This denotes the
	// key index within as a child of the account and branch.
	index: u32,
}

// KeyScope represents a restricted key scope from the primary root key within
// the HD chain. From the root manager (m/) we can create a nearly arbitrary
// number of ScopedKeyManagers of key derivation path: m/purpose'/cointype'.
// These scoped managers can then me managed indecently, as they house the
// encrypted cointype key and can derive any child keys from there on.
struct KeyScope {
	// purpose is the purpose of this key scope. This is the first child of
	// the master HD key.
	purpose: u32,

	// coin is a value that represents the particular coin which is the
	// child of the purpose key. With this key, any accounts, or other
	// children can be derived at all.
	coin: u32,
}