use bitcoin::util::bip32::{ExtendedPrivKey, ChildNumber};
use secp256k1::Secp256k1;

use std::error::Error;

use key_manager::{KeyManager, KeyScope};
use account_manager::AccountManager;

pub struct ScopedManager {
    scoped_key: ExtendedPrivKey,
}

impl ScopedManager {
    pub fn from_scoped_key(scoped_key: ExtendedPrivKey) -> Self {
        Self { scoped_key }
    }

    pub fn account_manager(&self, account: ChildNumber) -> Result<AccountManager, Box<Error>> {
        let path: &[ChildNumber] = &[
            account
        ];

        let account_key = self.scoped_key.derive_priv(&Secp256k1::new(),  &path)?;
        Ok(AccountManager::from_account_key(account_key))
    }
}