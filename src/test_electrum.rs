use bdk::{electrum_client::{Client, ElectrumApi}, bitcoin::{blockdata::script, Script, Address, Network}};

pub fn get_tx_history_address(script: &Script) -> usize {
    let client = Client::new("ssl://electrum.blockstream.info:60002").unwrap();
    // let res = client.server_features();
    // println!("{:#?}", res);
    let history_list = client.script_get_history(script).unwrap();
    return history_list.len();
}

pub fn get_address_balance(script: &Script) -> u64 {
    let client = Client::new("ssl://electrum.blockstream.info:60002").unwrap();
    // let res = client.server_features();
    // println!("{:#?}", res);
    let balance_res = client.script_get_balance(script).unwrap();
    return balance_res.confirmed;
}

pub struct AdditionalAddrInfo {
    pub index: u64,
    pub address: String,
    pub tx_count: u64,
    pub balance: u64
}

pub fn get_batch_history_and_balance(scripts: &Vec::<Script>) -> Vec::<AdditionalAddrInfo> {
    let client = Client::new("ssl://electrum.blockstream.info:60002").unwrap();

    let history_list = client.batch_script_get_history(scripts).unwrap();
    let balance_list = client.batch_script_get_balance(scripts).unwrap();

    let mut result = Vec::<AdditionalAddrInfo>::new();

    for n in 0..scripts.len() {
        let index: u64 = n.try_into().expect("cannot convert");
        let script = &scripts[n];
        let address = Address::from_script(&script, Network::Testnet).unwrap().to_string();
        let balance = balance_list[n].confirmed;
        let tx_count = history_list[n].len().try_into().expect("cannot convert");;

        result.push(AdditionalAddrInfo {
            index,
            address,
            tx_count,
            balance
        });

    }

    return result;
}