use async_std::task::block_on;
use bdk::{Wallet, Error};
use bdk::bitcoin::{Network, Script, Address};
use bdk::blockchain::ElectrumBlockchain;
use bdk::database::MemoryDatabase;
use bdk::electrum_client::ElectrumApi;
use bdk::wallet::{AddressIndex, AddressInfo};
use iced::{button,text_input, Application, executor, Command, Clipboard, Element, Text, Settings, TextInput, Length, Column, Button, Scrollable, Container, scrollable, Row, Align, window, Font};
use iced::HorizontalAlignment;

use bdk::miniscript::descriptor::DescriptorTrait;


use bdk::descriptor::derived::AsDerived;



use std::collections::HashSet;
use std::hash::Hash;
use std::option;
use std::str::FromStr;

mod w_electrum;
mod test_electrum;

const ROBOTO: Font = Font::External {
    name: "RobotoMono-Regular",
    bytes: include_bytes!("../fonts/RobotoMono-Regular.ttf"),
};

const ROBOTO_BOLD: Font = Font::External {
    name: "RobotoMono-Bold",
    bytes: include_bytes!("../fonts/RobotoMono-Bold.ttf"),
};

pub fn main() -> iced::Result {

    RuWallet::run(Settings {
        window: window::Settings {
            size: (1600, 768),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug, Default)]
struct AddressRow {
    index: u64,
    address: String,
    balance: u64,
    tx_count: u64
}

#[derive(Debug, Default)]
struct UTXORow {
    txid: String,
    vout: u32,
    address: String,
    amount: u64,
    height: u32
}

#[derive(Debug, Default, Clone)]
struct TransactionRow {
    txid: String,
    amount: i128,
    height: u32
}

#[derive(Debug, Default)]
struct RuWallet{
    scroll: scrollable::State,

    external_descriptor_input_state: text_input::State,
    external_descriptor_input_value: String,

    internal_descriptor_input_state: text_input::State,
    internal_descriptor_input_value: String,

    create_wallet_button_state: button::State,

    new_address: String,

    address_items: Vec<AddressRow>,

    internal_address_items: Vec<AddressRow>,

    utxo_items: Vec<UTXORow>,

    transaction_items: Vec<TransactionRow>
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
        String::from("Rust Wallet")
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
                println!("{}", "Restoring wallet ...");

                self.address_items.clear();
                self.new_address.clear();
                self.internal_address_items.clear();
                self.utxo_items.clear();
                self.transaction_items.clear();

                let wallet = block_on(self.generate_wallet());

                self.address_items = block_on(self.get_external_addresses(&wallet));

                self.internal_address_items = block_on(self.get_internal_addresses(&wallet));

                self.new_address = wallet.get_address(AddressIndex::New).unwrap().address.to_string();

                let mut tx_list = wallet.list_transactions(true).unwrap();

                tx_list.sort_by(|a, b|
                    b.confirmation_time.as_ref().unwrap().height.cmp(&a.confirmation_time.as_ref().unwrap().height));

                for tx in tx_list.iter() {

                    let height = tx.confirmation_time.as_ref().unwrap().height;

                    let amount = tx.received as i128 - tx.sent as i128;

                    self.transaction_items.push(
                        TransactionRow {
                            txid: tx.txid.to_string(),
                            amount,
                            height
                        }
                    );
                }

                for utxo in wallet.list_unspent().unwrap().iter() {

                    let addr = Address::from_script(&utxo.txout.script_pubkey, Network::Testnet).unwrap();

                    let utxo_tx = (&mut self.transaction_items).iter_mut().find(
                        |tr| tr.txid.to_string().eq(&utxo.outpoint.txid.to_string())
                    );

                    let height = match utxo_tx {
                        Some(tr) => tr.height,
                        None => 0,
                    };

                    self.utxo_items.push(
                        UTXORow {
                            txid: utxo.outpoint.txid.to_string(),
                            vout: utxo.outpoint.vout,
                            address: addr.to_string(),
                            amount: utxo.txout.value,
                            height
                        }
                    )
                }

                self.utxo_items.sort_by(|a, b| b.height.cmp(&a.height));
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {

        let title = Text::new("Rust Wallet")
            .font(ROBOTO_BOLD)
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
        .size(20)
        .font(ROBOTO);
        //.on_submit(Self::Message::CreateWallet);

        let internal_descriptor_input = TextInput::new(
            &mut self.internal_descriptor_input_state,
            "Enter Internal Descriptor",
            &mut self.internal_descriptor_input_value,
            Self::Message::InternalDescriptorInputChanged
        )
        .padding(15)
        .size(20)
        .font(ROBOTO);
        //.on_submit(Self::Message::CreateWallet);

        let create_wallet_button = Button::new(
            &mut self.create_wallet_button_state,
            Text::new("Restore Wallet")
        )
        .padding(15)
        .on_press(Self::Message::CreateWallet);

        let mut content = Column::new()
            .spacing(20)
            .push(title)
            .push(external_descriptor_input)
            .push(internal_descriptor_input)
            .push(create_wallet_button);

        if !self.address_items.is_empty() {

            let address_list_title = Text::new("Address List")
                .font(ROBOTO_BOLD)
                .width(Length::Fill)
                .size(35)
                .color([0.5, 0.5, 0.5])
                .horizontal_alignment(HorizontalAlignment::Left);

            let mut address_table: Column<RuWalletMessage> = Column::new()
                .width(iced::Length::FillPortion(1000))
                .spacing(10);


            let mut table_header: Row<RuWalletMessage> = Row::new()
                .align_items(Align::Start)
                .spacing(10);

            table_header = table_header
                .push(
                    Text::new("Index")
                        .font(ROBOTO)
                        .width(Length::Units(50))
                        .size(18)
                        .horizontal_alignment(HorizontalAlignment::Left)
                )
                .push(
                    Text::new("Type")
                        .font(ROBOTO)
                        .width(Length::Units(110))
                        .size(18)
                        .horizontal_alignment(HorizontalAlignment::Left)
                )
                .push(
                    Text::new("Address")
                        .font(ROBOTO)
                        .width(Length::Units(410))
                        .size(18)
                        .horizontal_alignment(HorizontalAlignment::Left)
                )
                .push(
                    Text::new("Balance (sats)")
                        .font(ROBOTO)
                        .width(Length::Units(150))
                        .size(18)
                        .horizontal_alignment(HorizontalAlignment::Left)
                )
                .push(
                    Text::new("Tx Count")
                        .font(ROBOTO)
                        .width(Length::Units(90))
                        .size(18)
                        .horizontal_alignment(HorizontalAlignment::Left)
                );

            address_table = address_table.push(table_header);


            for addr_item in &self.address_items {

                let mut table_row = Row::new()
                    .align_items(Align::Start)
                    .spacing(10);

                let addr_index_text = Text::new(addr_item.index.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(50))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Left);

                let addr_type_text = Text::new("receiving")
                    .font(ROBOTO)
                    .width(Length::Units(110))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Left);

                let addr_text = Text::new(addr_item.address.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(410))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Left);

                let addr_balance_text = Text::new(addr_item.balance.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(150))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Right);

                let addr_tx_count_text = Text::new(addr_item.tx_count.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(90))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Right);

                table_row = table_row
                    .push(addr_index_text)
                    .push(addr_type_text)
                    .push(addr_text)
                    .push(addr_balance_text)
                    .push(addr_tx_count_text);

                address_table = address_table.push(table_row);

            }


            for addr_item in &self.internal_address_items {

                let mut table_row = Row::new()
                    .align_items(Align::Start)
                    .spacing(10);

                let addr_index_text = Text::new(addr_item.index.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(50))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Left);

                let addr_type_text = Text::new("change")
                    .font(ROBOTO)
                    .width(Length::Units(110))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Left);

                let addr_text = Text::new(addr_item.address.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(410))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Left);

                let addr_balance_text = Text::new(addr_item.balance.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(150))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Right);

                let addr_tx_count_text = Text::new(addr_item.tx_count.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(90))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Right);

                table_row = table_row
                    .push(addr_index_text)
                    .push(addr_type_text)
                    .push(addr_text)
                    .push(addr_balance_text)
                    .push(addr_tx_count_text);

                address_table = address_table.push(table_row);

            }

            content = content
                .push(address_list_title)
                .push(address_table);
        }

        // show new address
        if !self.new_address.is_empty() {

            let new_address_title = Text::new("Current Receive Address")
                .font(ROBOTO_BOLD)
                .width(Length::Fill)
                .size(35)
                .color([0.5, 0.5, 0.5])
                .horizontal_alignment(HorizontalAlignment::Left);

            let new_address_text =Text::new(&self.new_address)
                .font(ROBOTO)
                .width(Length::Fill)
                .size(20)
                .horizontal_alignment(HorizontalAlignment::Left);

            content = content
                .push(new_address_title)
                .push(new_address_text);
        }

        if !self.utxo_items.is_empty() {

            let unspent_list_title = Text::new("Unspent List")
                .font(ROBOTO_BOLD)
                .width(Length::Fill)
                .size(35)
                .color([0.5, 0.5, 0.5])
                .horizontal_alignment(HorizontalAlignment::Left);

            let mut unspent_table: Column<RuWalletMessage> = Column::new()
                .width(iced::Length::Fill)
                .spacing(10);

            let mut table_header: Row<RuWalletMessage> = Row::new()
                .align_items(Align::Start)
                .spacing(10);

            table_header = table_header
                .push(
                    Text::new("Output Point")
                        .font(ROBOTO_BOLD)
                        .width(Length::Units(610))
                        .size(18)
                        .horizontal_alignment(HorizontalAlignment::Left)
                )
                .push(
                    Text::new("Address")
                        .font(ROBOTO_BOLD)
                        .width(Length::Units(410))
                        .size(18)
                        .horizontal_alignment(HorizontalAlignment::Left)
                )
                .push(
                    Text::new("Amount (sats)")
                        .font(ROBOTO_BOLD)
                        .width(Length::Units(150))
                        .size(18)
                        .horizontal_alignment(HorizontalAlignment::Right)
                )
                .push(
                    Text::new("Height")
                        .font(ROBOTO_BOLD)
                        .width(Length::Units(110))
                        .size(18)
                        .horizontal_alignment(HorizontalAlignment::Right)
                );

            unspent_table = unspent_table.push(table_header);

            for utxo_item in &self.utxo_items {

                let mut table_row: Row<RuWalletMessage> = Row::new()
                    .align_items(Align::Start)
                    .spacing(10);

                let txid_vout = format!("{}:{}", utxo_item.txid.to_string(), utxo_item.vout);

                let txid_text = Text::new(txid_vout)
                    .font(ROBOTO)
                    .width(Length::Units(610))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Left);

                let address_text = Text::new(&utxo_item.address)
                    .font(ROBOTO)
                    .width(Length::Units(410))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Left);

                let address_amount = Text::new(utxo_item.amount.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(150))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Right);

                let height = Text::new(utxo_item.height.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(110))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Right);

                table_row = table_row
                    .push(txid_text)
                    .push(address_text)
                    .push(address_amount)
                    .push(height);

                unspent_table = unspent_table.push(table_row);
            }

            content = content
                .push(unspent_list_title)
                .push(unspent_table);
        }

        if !self.transaction_items.is_empty() {

            let tx_list_title = Text::new("Transaction List")
                .font(ROBOTO_BOLD)
                .width(Length::Fill)
                .size(35)
                .color([0.5, 0.5, 0.5])
                .horizontal_alignment(HorizontalAlignment::Left);

            let mut transaaction_table: Column<RuWalletMessage> = Column::new()
                .width(iced::Length::Fill)
                .spacing(10);

            let mut table_header: Row<RuWalletMessage> = Row::new()
                .align_items(Align::Start)
                .spacing(10);

            table_header = table_header
                .push(
                    Text::new("Transaction Id")
                        .font(ROBOTO_BOLD)
                        .width(Length::Units(610))
                        .size(20)
                        .horizontal_alignment(HorizontalAlignment::Left)
                )
                .push(
                    Text::new("Amount (sats)")
                        .font(ROBOTO_BOLD)
                        .width(Length::Units(150))
                        .size(20)
                        .horizontal_alignment(HorizontalAlignment::Right)
                )
                .push(
                    Text::new("Height")
                        .font(ROBOTO_BOLD)
                        .width(Length::Units(110))
                        .size(20)
                        .horizontal_alignment(HorizontalAlignment::Right)
                );

            transaaction_table = transaaction_table.push(table_header);

            for transaction_item in &self.transaction_items {

                let mut table_row: Row<RuWalletMessage> = Row::new()
                    .align_items(Align::Start)
                    .spacing(10);

                let txid_text = Text::new(&transaction_item.txid)
                    .font(ROBOTO)
                    .width(Length::Units(610))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Left);

                let amount_text = Text::new(transaction_item.amount.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(150))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Right);

                let height_amount = Text::new(transaction_item.height.to_string())
                    .font(ROBOTO)
                    .width(Length::Units(110))
                    .size(20)
                    .horizontal_alignment(HorizontalAlignment::Right);

                table_row = table_row
                    .push(txid_text)
                    .push(amount_text)
                    .push(height_amount);

                transaaction_table = transaaction_table.push(table_row);
            }

            content = content
                .push(tx_list_title)
                .push(transaaction_table);
        }

        Scrollable::new(&mut self.scroll)
            .padding(40)
            .width(Length::Units(13400))
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
            &self.internal_descriptor_input_value
        );

        wallet
    }

    async fn get_external_addresses(&self, wallet: &Wallet<ElectrumBlockchain, MemoryDatabase>) -> Vec::<AddressRow> {

        let electrum_url = "ssl://electrum.blockstream.info:60002";

        let mut scripts = Vec::<Script>::new();

        for n in 0..10 {
            let address_info = wallet.get_address(AddressIndex::Peek(n)).unwrap();

            scripts.push(address_info.script_pubkey());

        }

        let additional_addr_info =
            w_electrum::get_batch_history_and_balance(electrum_url, &scripts);

        let mut result = Vec::<AddressRow>::new();

        for aai in additional_addr_info {
            result.push(
                AddressRow {
                    index: aai.index,
                    address: aai.address,
                    balance: aai.balance,
                    tx_count: aai.tx_count
                }
            );
        }

        result
    }

    async fn get_internal_addresses(&self, wallet: &Wallet<ElectrumBlockchain, MemoryDatabase>) -> Vec::<AddressRow> {

        let electrum_url = "ssl://electrum.blockstream.info:60002";

        let mut scripts = Vec::<Script>::new();

        for n in 0..10 {
            let address_info = self.peek_change_address(wallet, n).unwrap();

            scripts.push(address_info.script_pubkey());

        }

        let additional_addr_info =
            w_electrum::get_batch_history_and_balance(electrum_url, &scripts);

        let mut result = Vec::<AddressRow>::new();

        for aai in additional_addr_info {
            result.push(
                AddressRow {
                    index: aai.index,
                    address: aai.address,
                    balance: aai.balance,
                    tx_count: aai.tx_count
                }
            );
        }

        result
    }


    fn peek_change_address(&self, wallet: &Wallet<ElectrumBlockchain, MemoryDatabase>, index: u32) -> Result<AddressInfo, Error> {

        let result_descriptor = wallet.get_descriptor_for_keychain(bdk::KeychainKind::Internal);

        result_descriptor
            .as_derived(index, wallet.secp_ctx())
            .address(wallet.network())
            .map(|address| AddressInfo { index, address })
            .map_err(|_| Error::ScriptDoesntHaveAddressForm)
    }
}