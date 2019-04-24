use std::path::Path;
use implementation::wallet_lib::{interface::Wallet, error::WalletError};

pub fn create_wallet<P>(db_path: &P) -> Result<Box<dyn Wallet + Send>, WalletError>
where
    P: AsRef<Path>,
{
    use implementation::wallet_lib::{
        walletlibrary::{DEFAULT_SALT, WalletConfig, WalletLibraryMode, KeyGenConfig},
        electrumx::ElectrumxWallet,
    };
    use bitcoin::network::constants::Network;
    //use std::io::stdin;

    //println!("enter password for wallet");
    //let mut passphrase = String::new();
    //stdin().read_line(&mut passphrase).unwrap();
    let passphrase = "qwerty".to_owned();

    let config = |passphrase: String| WalletConfig::new(
        Network::Regtest,
        passphrase,
        DEFAULT_SALT.to_owned(),
        db_path.as_ref().to_str().unwrap().to_owned(),
    );
    let wallet = ElectrumxWallet::new(config(passphrase.clone()), WalletLibraryMode::Decrypt)
        .map(|(wallet, _)| wallet)
        .or_else(|e| match e {
            WalletError::HasNoWalletInDatabase => {
                let mode = WalletLibraryMode::Create(KeyGenConfig::default());
                ElectrumxWallet::new(config(passphrase), mode)
                    .map(|(wallet, mnemonic)| {
                        println!("{}", mnemonic.to_string());
                        wallet
                    })
            },
            e @ _ => Err(e),
        })?;

    Ok(Box::new(wallet))
}
