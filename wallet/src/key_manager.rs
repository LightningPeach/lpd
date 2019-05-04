use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey, ChildNumber};
use bitcoin::network::constants::Network;
use secp256k1::Secp256k1;

use std::error::Error;

use scoped_manager::ScopedManager;

pub struct KeyManager {
    master_key: ExtendedPrivKey,
}

impl KeyManager {
    pub fn from_seed(seed: &[u8]) -> Result<Self, Box<Error>> {
        Ok(Self {
            master_key: ExtendedPrivKey::new_master(Network::Bitcoin, seed)?,
        })
    }

    pub fn scoped_manager(&self, scope: &KeyScope) -> Result<ScopedManager, Box<Error>> {
        let path: &[ChildNumber] = &[
            scope.purpose,
            scope.coin,
        ];

        let scoped_key = self.master_key.derive_priv(&Secp256k1::new(), &path)?;
        Ok(ScopedManager::from_scoped_key(scoped_key))
    }

    // TODO(evg): use another key derivation scheme?
//    pub fn derive_public_key_from_path(&self, key_scope: &KeyScope, derivation_path: &DerivationPath) -> Result<ExtendedPubKey, Box<Error>> {
//        let extended_priv_key = self.derive_private_key_from_path(key_scope, derivation_path)?;
//        Ok(ExtendedPubKey::from_private(&Secp256k1::new(), &extended_priv_key))
//    }
//
//    fn derive_private_key_from_path(&self, key_scope: &KeyScope, derivation_path: &DerivationPath) -> Result<ExtendedPrivKey, Box<Error>> {
//        let path: &[ChildNumber] = &[
//            key_scope.purpose,
//            key_scope.coin,
//
//            derivation_path.account,
//            derivation_path.branch,
//            derivation_path.index,
//        ];
//
//        let derived_key = ExtendedPrivKey::from_path(&Secp256k1::new(), &self.master_key, path)?;
//        Ok(derived_key)
//    }
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
//pub struct DerivationPath {
//	// account is the account, or the first immediate child from the scoped
//	// manager's hardened coin type key.
//	pub account: ChildNumber,
//
//	// branch is the branch to be derived from the account index above. For
//	// BIP0044-like derivation, this is either 0 (external) or 1
//	// (internal). However, we allow this value to vary arbitrarily within
//	// its size range.
//	pub branch: ChildNumber,
//
//	// index is the final child in the derivation path. This denotes the
//	// key index within as a child of the account and branch.
//	pub index: ChildNumber,
//}

// KeyScope represents a restricted key scope from the primary root key within
// the HD chain. From the root manager (m/) we can create a nearly arbitrary
// number of ScopedKeyManagers of key derivation path: m/purpose'/cointype'.
// These scoped managers can then me managed indecently, as they house the
// encrypted cointype key and can derive any child keys from there on.
pub struct KeyScope {
    // purpose is the purpose of this key scope. This is the first child of
    // the master HD key.
    pub purpose: ChildNumber,

    // coin is a value that represents the particular coin which is the
    // child of the purpose key. With this key, any accounts, or other
    // children can be derived at all.
    pub coin: ChildNumber,
}