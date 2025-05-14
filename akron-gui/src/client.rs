use std::time::Duration;

use iced::Task;
use jsonrpsee::{
    core::ClientError,
    http_client::{HttpClient, HttpClientBuilder},
};

use spaces_client::{
    config::default_spaces_rpc_port,
    config::ExtendedNetwork,
    rpc::{
        BidParams, OpenParams, RegisterParams, RpcClient, RpcWalletRequest, RpcWalletTxBuilder,
        SendCoinsParams, TransferSpacesParams,
    },
};
use spaces_protocol::constants::ChainAnchor;

pub use spaces_client::{
    rpc::ServerInfo,
    wallets::{AddressKind, ListSpacesResponse, TxInfo, WalletInfoWithProgress, WalletResponse},
};
pub use spaces_protocol::{bitcoin::Txid, slabel::SLabel, Covenant, FullSpaceOut};
pub use spaces_wallet::{
    bitcoin::{Amount, FeeRate, OutPoint},
    export::WalletExport,
    nostr::NostrEvent,
    tx_event::{
        BidEventDetails, BidoutEventDetails, OpenEventDetails, SendEventDetails, TxEvent,
        TxEventKind,
    },
    Balance, Listing,
};

use akrond::{runner::ServiceKind, Akron};

use crate::ConfigBackend;

#[derive(Debug, Clone)]
pub struct Client {
    client: HttpClient,
    shutdown: Option<tokio::sync::broadcast::Sender<()>>,
    pub logs: Option<tokio::sync::broadcast::Sender<String>>,
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
    pub async fn create(
        data_dir: std::path::PathBuf,
        mut backend_config: ConfigBackend,
    ) -> Result<(Self, ConfigBackend), String> {
        let mut logs = None;
        // TODO: move this as a command line flag --no-capture-logs (uses stdout instead)
        const CAPTURE_LOGS : bool = true;
        let (spaces_rpc_url, shutdown) = match &mut backend_config {
            ConfigBackend::Akrond {
                network,
                prune_point,
            } => {
                let (akron, shutdown) = Akron::create(CAPTURE_LOGS);
                logs = akron.subscribe_logs();
                let yuki_data_dir = data_dir.join("yuki");
                let spaces_data_dir = data_dir.join("spaces");
                let mut yuki_args: Vec<String> = [
                    "--chain",
                    &network.to_string(),
                    "--data-dir",
                    yuki_data_dir.to_str().unwrap(),
                ]
                .iter()
                .map(|s| s.to_string())
                .collect();
                let spaces_args: Vec<String> = [
                    "--chain",
                    &network.to_string(),
                    "--bitcoin-rpc-url",
                    "http://127.0.0.1:8225",
                    "--data-dir",
                    spaces_data_dir.to_str().unwrap(),
                    "--bitcoin-rpc-light",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect();
                if prune_point.is_none() {
                    match network {
                        ExtendedNetwork::Mainnet => {
                            let checkpoint = akron
                                .load_checkpoint(
                                    "https://bitpki.com/protocol.sdb",
                                    &spaces_data_dir.join(network.to_string()),
                                    None,
                                )
                                .await
                                .map_err(|e| e.to_string())?;

                            *prune_point = Some(checkpoint.block);
                        }
                        ExtendedNetwork::Testnet4 => *prune_point = Some(ChainAnchor::TESTNET4()),
                        _ => {}
                    }
                }
                if let Some(prune_point) = prune_point {
                    yuki_args.push("--prune-point".to_string());
                    yuki_args.push(format!(
                        "{}:{}",
                        hex::encode(prune_point.hash),
                        prune_point.height
                    ));
                }
                yuki_args.push("--filters-endpoint".to_string());
                yuki_args.push("https://bitpki.com/".to_string());

                if let Err(e) = akron.start(ServiceKind::Yuki, yuki_args).await {
                    let _ = shutdown.send(());
                    return Err(e.to_string());
                }
                if let Err(e) = akron
                    .start(
                        ServiceKind::Spaces,
                        spaces_args.iter().map(|s| s.to_string()).collect(),
                    )
                    .await
                {
                    let _ = shutdown.send(());
                    return Err(e.to_string());
                }
                (
                    format!("http://127.0.0.1:{}", default_spaces_rpc_port(network)),
                    Some(shutdown),
                )
            }
            ConfigBackend::Bitcoind {
                network,
                url,
                cookie,
                user,
                password,
            } => {
                let (akron, shutdown) = Akron::create(CAPTURE_LOGS);
                logs = akron.subscribe_logs();
                let spaces_data_dir = data_dir.join("spaces");
                let network_string = network.to_string();
                let mut spaces_args = vec![
                    "--chain",
                    &network_string,
                    "--data-dir",
                    spaces_data_dir.to_str().unwrap(),
                    "--bitcoin-rpc-url",
                    url,
                ];
                if !cookie.is_empty() {
                    spaces_args.extend_from_slice(&["--bitcoin-rpc-cookie", cookie]);
                }
                if !user.is_empty() {
                    spaces_args.extend_from_slice(&[
                        "--bitcoin-rpc-user",
                        user,
                        "--bitcoin-rpc-password",
                        password,
                    ]);
                }
                if let Err(e) = akron
                    .start(
                        ServiceKind::Spaces,
                        spaces_args.iter().map(|s| s.to_string()).collect(),
                    )
                    .await
                {
                    let _ = shutdown.send(());
                    return Err(e.to_string());
                }
                (
                    format!("http://127.0.0.1:{}", default_spaces_rpc_port(network)),
                    Some(shutdown),
                )
            }
            ConfigBackend::Spaced { url, .. } => (url.to_string(), None),
        };
        let client = HttpClientBuilder::default()
            .build(spaces_rpc_url)
            .map_err(|e| e.to_string())?;
        let mut server_info_result = client.get_server_info().await;
        let mut attempts = 1;
        while server_info_result.is_err() && attempts != 5 {
            server_info_result = client.get_server_info().await;
            let _ = tokio::time::sleep(Duration::from_secs(1)).await;
            attempts += 1;
        }
        match server_info_result {
            Ok(server_info) => {
                match &backend_config {
                    ConfigBackend::Akrond { .. } => {}
                    ConfigBackend::Bitcoind { network, .. }
                    | ConfigBackend::Spaced { network, .. } => {
                        if server_info.network != network.to_string() {
                            if let Some(shutdown) = shutdown {
                                let _ = shutdown.send(());
                            }
                            return Err("Wrong network".to_string());
                        }
                    }
                };
            }
            Err(e) => {
                if let Some(shutdown) = shutdown {
                    let _ = shutdown.send(());
                }
                return Err(e.to_string());
            }
        }
        Ok((Self { client, shutdown, logs }, backend_config))
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

impl Drop for Client {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.as_ref() {
            let _ = shutdown.send(());
        }
    }
}
