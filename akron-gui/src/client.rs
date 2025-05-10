use iced::Task;
use jsonrpsee::{
    core::ClientError,
    http_client::{HttpClient, HttpClientBuilder},
};

use spaces_client::rpc::{
    BidParams, OpenParams, RegisterParams, RpcClient, RpcWalletRequest, RpcWalletTxBuilder,
    SendCoinsParams, TransferSpacesParams,
};

pub use spaces_client::{
    rpc::ServerInfo,
    wallets::{AddressKind, ListSpacesResponse, TxInfo, WalletInfoWithProgress, WalletResponse},
};
pub use spaces_protocol::{Covenant, FullSpaceOut, bitcoin::Txid, slabel::SLabel};
pub use spaces_wallet::{
    Balance, Listing,
    bitcoin::{Amount, FeeRate, OutPoint},
    export::WalletExport,
    nostr::NostrEvent,
    tx_event::{
        BidEventDetails, BidoutEventDetails, OpenEventDetails, SendEventDetails, TxEvent,
        TxEventKind,
    },
};

#[derive(Debug)]
pub struct Client {
    client: HttpClient,
}

pub type ClientResult<T> = Result<T, String>;

fn map_result<T>(result: Result<T, ClientError>) -> ClientResult<T> {
    result.map_err(|e| match e {
        ClientError::Call(e) => e.message().to_string(),
        _ => e.to_string(),
    })
}

#[derive(Debug, Clone)]
pub struct WalletResult<T> {
    pub label: String,
    pub result: Result<T, String>,
}

fn map_wallet_result<T>((label, result): (String, Result<T, ClientError>)) -> WalletResult<T> {
    WalletResult {
        label,
        result: map_result(result),
    }
}

impl Client {
    pub fn new(rpc_url: &str) -> Result<Self, String> {
        let client = HttpClientBuilder::default()
            .build(rpc_url)
            .map_err(|e| e.to_string())?;
        Ok(Self { client })
    }

    pub fn get_server_info(&self) -> Task<ClientResult<ServerInfo>> {
        let client = self.client.clone();
        Task::perform(async move { client.get_server_info().await }, map_result)
    }

    pub fn get_space_info(
        &self,
        slabel: SLabel,
    ) -> Task<ClientResult<(SLabel, Option<FullSpaceOut>)>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                use spaces_client::store::Sha256;
                use spaces_protocol::hasher::KeyHasher;
                let hash = hex::encode(Sha256::hash(slabel.as_ref()));
                let result = client.get_space(&hash).await;
                result.map(|r| (slabel, r))
            },
            map_result,
        )
    }

    pub fn list_wallets(&self) -> Task<ClientResult<Vec<String>>> {
        let client = self.client.clone();
        Task::perform(async move { client.list_wallets().await }, map_result)
    }

    pub fn create_wallet(&self, wallet: String) -> Task<WalletResult<()>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_create(&wallet).await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn load_wallet(&self, wallet: String) -> Task<WalletResult<()>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_load(&wallet).await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn export_wallet(&self, wallet: String) -> Task<WalletResult<String>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_export(&wallet).await;
                (wallet, result.map(|w| w.to_string()))
            },
            map_wallet_result,
        )
    }

    pub fn import_wallet(&self, wallet_string: &str) -> Task<Result<String, String>> {
        let wallet_export: Result<WalletExport, _> = std::str::FromStr::from_str(wallet_string);
        match wallet_export {
            Ok(wallet_export) => {
                let client = self.client.clone();
                Task::perform(
                    async move {
                        let label = wallet_export.label.clone();
                        let result = client.wallet_import(wallet_export).await;
                        result.map(|_| label)
                    },
                    map_result,
                )
            }
            Err(err) => Task::done(Err(err.to_string())),
        }
    }

    pub fn get_wallet_info(&self, wallet: String) -> Task<WalletResult<WalletInfoWithProgress>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_get_info(&wallet).await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn get_wallet_balance(&self, wallet: String) -> Task<WalletResult<Balance>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_get_balance(&wallet).await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn get_wallet_spaces(&self, wallet: String) -> Task<WalletResult<ListSpacesResponse>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_list_spaces(&wallet).await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn get_wallet_transactions(
        &self,
        wallet: String,
        count: usize,
    ) -> Task<WalletResult<Vec<TxInfo>>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_list_transactions(&wallet, count, 0).await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn get_wallet_address(
        &self,
        wallet: String,
        address_kind: AddressKind,
    ) -> Task<WalletResult<(AddressKind, String)>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_get_new_address(&wallet, address_kind).await;
                (wallet, result.map(|r| (address_kind, r)))
            },
            map_wallet_result,
        )
    }

    pub fn send_coins(
        &self,
        wallet: String,
        recipient: String,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    ) -> Task<WalletResult<WalletResponse>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client
                    .wallet_send_request(
                        &wallet,
                        RpcWalletTxBuilder {
                            bidouts: None,
                            requests: vec![RpcWalletRequest::SendCoins(SendCoinsParams {
                                amount,
                                to: recipient,
                            })],
                            fee_rate,
                            dust: None,
                            force: false,
                            confirmed_only: false,
                            skip_tx_check: false,
                        },
                    )
                    .await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn open_space(
        &self,
        wallet: String,
        slabel: SLabel,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    ) -> Task<WalletResult<WalletResponse>> {
        let name = slabel.to_string();
        let amount = amount.to_sat();
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client
                    .wallet_send_request(
                        &wallet,
                        RpcWalletTxBuilder {
                            bidouts: None,
                            requests: vec![RpcWalletRequest::Open(OpenParams { name, amount })],
                            fee_rate,
                            dust: None,
                            force: false,
                            confirmed_only: false,
                            skip_tx_check: false,
                        },
                    )
                    .await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn bid_space(
        &self,
        wallet: String,
        slabel: SLabel,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    ) -> Task<WalletResult<WalletResponse>> {
        let name = slabel.to_string();
        let amount = amount.to_sat();
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client
                    .wallet_send_request(
                        &wallet,
                        RpcWalletTxBuilder {
                            bidouts: None,
                            requests: vec![RpcWalletRequest::Bid(BidParams { name, amount })],
                            fee_rate,
                            dust: None,
                            force: false,
                            confirmed_only: false,
                            skip_tx_check: false,
                        },
                    )
                    .await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn register_space(
        &self,
        wallet: String,
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    ) -> Task<WalletResult<WalletResponse>> {
        let name = slabel.to_string();
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client
                    .wallet_send_request(
                        &wallet,
                        RpcWalletTxBuilder {
                            bidouts: None,
                            requests: vec![RpcWalletRequest::Register(RegisterParams {
                                name,
                                to: None,
                            })],
                            fee_rate,
                            dust: None,
                            force: false,
                            confirmed_only: false,
                            skip_tx_check: false,
                        },
                    )
                    .await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn renew_space(
        &self,
        wallet: String,
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    ) -> Task<WalletResult<WalletResponse>> {
        let name = slabel.to_string();
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client
                    .wallet_send_request(
                        &wallet,
                        RpcWalletTxBuilder {
                            bidouts: None,
                            requests: vec![RpcWalletRequest::Transfer(TransferSpacesParams {
                                spaces: vec![name],
                                to: None,
                            })],
                            fee_rate,
                            dust: None,
                            force: false,
                            confirmed_only: false,
                            skip_tx_check: false,
                        },
                    )
                    .await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn send_space(
        &self,
        wallet: String,
        recipient: String,
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    ) -> Task<WalletResult<WalletResponse>> {
        let name = slabel.to_string();
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client
                    .wallet_send_request(
                        &wallet,
                        RpcWalletTxBuilder {
                            bidouts: None,
                            requests: vec![RpcWalletRequest::Transfer(TransferSpacesParams {
                                spaces: vec![name],
                                to: Some(recipient),
                            })],
                            fee_rate,
                            dust: None,
                            force: false,
                            confirmed_only: false,
                            skip_tx_check: false,
                        },
                    )
                    .await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn bump_fee(
        &self,
        wallet: String,
        txid: Txid,
        fee_rate: FeeRate,
    ) -> Task<WalletResult<()>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_bump_fee(&wallet, txid, fee_rate, false).await;
                (wallet, result.map(|_| ()))
            },
            map_wallet_result,
        )
    }

    pub fn buy_space(
        &self,
        wallet: String,
        listing: Listing,
        fee_rate: Option<FeeRate>,
    ) -> Task<WalletResult<()>> {
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_buy(&wallet, listing, fee_rate, false).await;
                (wallet, result.map(|_| ()))
            },
            map_wallet_result,
        )
    }

    pub fn sell_space(
        &self,
        wallet: String,
        slabel: SLabel,
        price: Amount,
    ) -> Task<WalletResult<Listing>> {
        let client = self.client.clone();
        let space = slabel.to_string();
        let amount = price.to_sat();
        Task::perform(
            async move {
                let result = client.wallet_sell(&wallet, space, amount).await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }

    pub fn sign_event(
        &self,
        wallet: String,
        slabel: SLabel,
        event: NostrEvent,
    ) -> Task<WalletResult<NostrEvent>> {
        let space = slabel.to_string();
        let client = self.client.clone();
        Task::perform(
            async move {
                let result = client.wallet_sign_event(&wallet, &space, event).await;
                (wallet, result)
            },
            map_wallet_result,
        )
    }
}
