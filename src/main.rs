use async_std::task::block_on;
use bdk::Wallet;
use bdk::bitcoin::{Network, Script, Address};
use bdk::blockchain::ElectrumBlockchain;
use bdk::database::MemoryDatabase;
use bdk::electrum_client::ElectrumApi;
use bdk::wallet::AddressIndex;
use iced::{button,text_input, Application, executor, Command, Clipboard, Element, Text, Settings, TextInput, Length, Column, Button, Scrollable, Container, scrollable};
use iced::HorizontalAlignment;

mod w_electrum;

use std::hash::Hash;
use std::str::FromStr;

pub fn main() -> iced::Result {
    RuWallet::run(Settings::default())
}

/*
#[derive(Debug, Default)]
struct RWallet {
    input: text_input::State,
    input_value: String
}

*/

#[derive(Debug, Default)]
struct RuWallet{
    scroll: scrollable::State,

    external_descriptor_input_state: text_input::State,
    external_descriptor_input_value: String,

    internal_descriptor_input_state: text_input::State,
    internal_descriptor_input_value: String,

    create_wallet_button_state: button::State,

    used_address_items: Vec<String>,

    new_address: String,

    utxo_items: Vec<String>,

    transaction_items: Vec<String>
}

#[derive(Debug, Clone)]
enum RuWalletMessage {
    ExternalDescriptorInputChanged(String),
    InternalDescriptorInputChanged(String),
    CreateWallet,
}

impl Application for RuWallet {
    type Executor = executor::Default;
    type Message = RuWalletMessage;
    type Flags = ();

    fn new(_flags: ()) -> (RuWallet, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("A cool application")
    }

    fn update(&mut self, _message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match _message {
            RuWalletMessage::ExternalDescriptorInputChanged(value) => {
                self.external_descriptor_input_value = value.clone();
            },
            RuWalletMessage::InternalDescriptorInputChanged(value) => {
                self.internal_descriptor_input_value = value.clone();
            },
            RuWalletMessage::CreateWallet => {
                println!("{}", "Generating wallet ...");

                self.used_address_items.clear();
                self.new_address.clear();
                self.utxo_items.clear();

                let wallet = block_on(self.generate_wallet());

                println!("{}", "Wallet generated ...");

                let new_address_info = wallet.get_address(AddressIndex::New).unwrap();

                for n in 0..new_address_info.index {
                    let address = wallet.get_address(AddressIndex::Peek(n)).unwrap().address;
                    // println!("address {}: {}", n, address);
                    self.used_address_items.push(address.to_string());
                }

                self.new_address = new_address_info.address.to_string();

                // println!("new_address {}", self.new_address);

                for utxo in wallet.list_unspent().unwrap().iter() {

                  //  let addr = Script::new_v0_wpkh(utxo.txout.script_pubkey.hash(state));

                    let addr = Address::from_script(&utxo.txout.script_pubkey, Network::Testnet).unwrap();
                    self.utxo_items.push(
                        format!("{}: {} sats", addr.to_string(), utxo.txout.value)
                    );

                    // println!("utxo {}: {}", addr, utxo.txout.value);
                }

                let tx_list = wallet.list_transactions(false).unwrap();

                for tx in tx_list.iter() {

                    println!("{}: {} sats received, {} sats sent", tx.txid, tx.received, tx.sent);

                    self.transaction_items.push(
                        format!("{}: {} sats received, {} sats sent", tx.txid, tx.received, tx.sent)
                    );
                }


            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {

        let title = Text::new("R Wallet")
                    .width(Length::Fill)
                    .size(100)
                    .color([0.5, 0.5, 0.5])
                    .horizontal_alignment(HorizontalAlignment::Center);

        let external_descriptor_input = TextInput::new(
            &mut self.external_descriptor_input_state,
            "Enter External Descriptor",
            &mut self.external_descriptor_input_value,
            Self::Message::ExternalDescriptorInputChanged
        )
        .padding(15)
        .size(30)
        .on_submit(Self::Message::CreateWallet);

        let internal_descriptor_input = TextInput::new(
            &mut self.internal_descriptor_input_state,
            "Enter Internal Descriptor",
            &mut self.internal_descriptor_input_value,
            Self::Message::InternalDescriptorInputChanged
        )
        .padding(15)
        .size(30)
        .on_submit(Self::Message::CreateWallet);

        let create_wallet_button = Button::new(
            &mut self.create_wallet_button_state,
            Text::new("Generate Wallet")
        )
        .padding(15)
        .on_press(Self::Message::CreateWallet);

        let mut content = Column::new()
            .max_width(800)
            .spacing(20)
            .push(title)
            .push(external_descriptor_input)
            .push(internal_descriptor_input)
            .push(create_wallet_button);


        // show used addresses
        if !self.used_address_items.is_empty() {

            let used_addresses_title = Text::new("Used Addresses")
                .width(Length::Fill)
                .size(45)
                .color([0.5, 0.5, 0.5])
                .horizontal_alignment(HorizontalAlignment::Left);

            let used_address_list = self.used_address_items
                .iter()
                .enumerate()
                .fold(Column::new().spacing(20), |column, (_i, address_item)| {
                    let address_text = Text::new(address_item)
                        .width(Length::Fill)
                        .size(20)
                        .color([0.5, 0.5, 0.5])
                        .horizontal_alignment(HorizontalAlignment::Left);

                    column.push(address_text)
                });

            content = content
                .push(used_addresses_title)
                .push(used_address_list);
        }

        // show new address
        if !self.new_address.is_empty() {

            let new_address_title = Text::new("New Address")
            .width(Length::Fill)
            .size(45)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Left);

            let new_address_text =Text::new(&self.new_address)
                .width(Length::Fill)
                .size(20)
                .color([0.5, 0.5, 0.5])
                .horizontal_alignment(HorizontalAlignment::Left);

            content = content
                .push(new_address_title)
                .push(new_address_text);
        }

        // show UTXO list
        if !self.utxo_items.is_empty() {

            let unspent_list_title = Text::new("Unspent List")
                .width(Length::Fill)
                .size(45)
                .color([0.5, 0.5, 0.5])
                .horizontal_alignment(HorizontalAlignment::Left);

            let unspent_list = self.utxo_items
                .iter()
                .enumerate()
                .fold(Column::new().spacing(20), |column, (_i, utxo_item)| {
                    let utxo_text = Text::new(utxo_item)
                        .width(Length::Fill)
                        .size(20)
                        .color([0.5, 0.5, 0.5])
                        .horizontal_alignment(HorizontalAlignment::Left);

                    column.push(utxo_text)
                });

            content = content
                .push(unspent_list_title)
                .push(unspent_list);
        }

        // show transaction list
        if !self.utxo_items.is_empty() {

            let tx_list_title = Text::new("Transaction List")
                .width(Length::Fill)
                .size(45)
                .color([0.5, 0.5, 0.5])
                .horizontal_alignment(HorizontalAlignment::Left);

            let tx_list = self.transaction_items
                .iter()
                .enumerate()
                .fold(Column::new().spacing(20), |column, (_i, tx_item)| {
                    let tx_text = Text::new(tx_item)
                        .width(Length::Fill)
                        .size(20)
                        .color([0.5, 0.5, 0.5])
                        .horizontal_alignment(HorizontalAlignment::Left);

                    column.push(tx_text)
                });

            content = content
                .push(tx_list_title)
                .push(tx_list);
        }


        Scrollable::new(&mut self.scroll)
            .padding(40)
            .push(
                Container::new(content).width(Length::Fill).center_x(),
            )
            .into()
    }


}

impl RuWallet {

    // additional function non-related to GUI

    async fn generate_wallet(&self) -> Wallet<ElectrumBlockchain, MemoryDatabase> {
        let network = Network::Testnet;

        let electrum_url = "ssl://electrum.blockstream.info:60002";

        let wallet = w_electrum::load_or_create_wallet(electrum_url,
            &network,
            &self.external_descriptor_input_value,
            &self.internal_descriptor_input_value,
            );

        wallet
    }
}

// Move to other file later
/*
#[derive(Debug, Clone)]
struct AddressItem {
    address: String
}

struct AddressItemMessage;

impl AddressItem {
    fn new(val: String) -> Self {
        AddressItem { address: val }
    }

    fn update(&mut self, _message: AddressItemMessage) { }

    fn view(&mut self) -> Element<AddressItemMessage> {
        let address = Text::new("R Wallet")
            .width(Length::Fill)
            .size(30)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Center);

        address.into()
    }
}
*/

/*
fn new(_flags: ()) -> (Hello, Command<Self::Message>) {
        (Hello, Command::none())
    }

    fn title(&self) -> String {
        String::from("A cool application")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Text::new("Hello, world!").into()
    }
*/