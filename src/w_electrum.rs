use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::util::bip32::{DerivationPath, KeySource, ExtendedPrivKey};
use bdk::descriptor::Segwitv0;
use bdk::keys::{ExtendedKey, DerivableKey, DescriptorKey, DescriptorKey::Secret};
use bdk::keys::bip39::Mnemonic;
use bdk::template::Bip84;
use bdk::wallet::export::WalletExport;
use bdk::{Wallet, SignOptions, KeychainKind};
use bdk::database::MemoryDatabase;
use bdk::blockchain::{noop_progress, ElectrumBlockchain};
use bdk::bitcoin::{Network, Address, Transaction};

use bdk::electrum_client::Client;
use bdk::wallet::AddressIndex;

use std::str::FromStr;

/*use crate::wallet_common::{get_descriptors, build_signed_tx, mnemonic_to_xprv};

pub fn load_or_create_wallet(electrum_url: &str, network: &Network, xpriv: &ExtendedPrivKey)  -> Wallet<ElectrumBlockchain, MemoryDatabase>
{
    // Apparently it works only with Electrs (not EletrumX)
    let client = Client::new(electrum_url).unwrap();

    let wallet = Wallet::new(
        Bip84(xpriv.clone(), KeychainKind::External),
        Some(Bip84(*xpriv, KeychainKind::Internal)),
        *network,
        MemoryDatabase::default(),
        ElectrumBlockchain::from(client)
    ).unwrap();

    wallet.sync(noop_progress(), None).unwrap();

    wallet
}
*/

pub fn load_or_create_wallet(electrum_url: &str, network: &Network, external_descriptor: &str, internal_descriptor: &str)  -> Wallet<ElectrumBlockchain, MemoryDatabase>
{
    // Apparently it works only with Electrs (not EletrumX)
    let client = Client::new(electrum_url).unwrap();

    let wallet = Wallet::new(
        external_descriptor,
        Some(internal_descriptor),
        *network,
        MemoryDatabase::default(),
        ElectrumBlockchain::from(client)
    ).unwrap();

    wallet.sync(noop_progress(), None).unwrap();

    wallet
}

pub fn run(network: Network, external_descriptor: &str, internal_descriptor: &str, electrum_url: &str) {

    //let xpriv = mnemonic_to_xprv(&network, &mnemonic_words);

    let wallet = load_or_create_wallet(electrum_url, &network, external_descriptor, internal_descriptor);

    for n in 0..10 {
        let address = wallet.get_address(AddressIndex::Peek(n)).unwrap().address;
        println!("address {}: {}", n, address);
    }

    /* let address = wallet.get_address(AddressIndex::New).unwrap().address;

    println!("address: {}", address);

    let balance = wallet.get_balance().unwrap();

    println!("balance: {}", balance);*/

    /*if balance > 100 {

        let recipient_address = "tb1qfhg76n90tc985rvwyw3cg3x6pnhqd32yddw3pw";

        let amount = 9359;

        let tx = build_signed_tx(&wallet, recipient_address, amount);

        let tx_id = wallet.broadcast(&tx).unwrap();

        println!("tx id: {}", tx_id.to_string());
    } else {
        println!("Insufficient Funds. Fund the wallet with the address above");
    }

    let export = WalletExport::export_wallet(&wallet, "exported wallet", true)
        .map_err(ToString::to_string)
        .map_err(bdk::Error::Generic).unwrap();

    println!("------\nWallet Backup: {}", export.to_string());*/

}