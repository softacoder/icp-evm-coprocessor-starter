use alloy::{
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    primitives::{utils::parse_ether, Address, U256},
    providers::{
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            WalletFiller,
        },
        Identity, Provider, ProviderBuilder, RootProvider,
    },
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
};
use alloy_node_bindings::{Anvil, AnvilInstance};
use reqwest::Url;
use serde_json::json;

pub type EvmProvider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider,
    Ethereum,
>;

pub struct EvmEnv {
    pub user: Address,
    pub contract: Address,
    pub provider: EvmProvider,
    pub anvil_url: Url,
    // Unused, but we need to keep it alive.
    _anvil_instance: AnvilInstance,
}

impl EvmEnv {
    pub async fn new() -> Self {
        let anvil_instance = Anvil::new().try_spawn().unwrap();
        let anvil_url: Url = anvil_instance.endpoint().parse().unwrap();
        let user_addr = anvil_instance.addresses()[0];
        let user_key = anvil_instance.keys()[0].clone();
        let signer: PrivateKeySigner = user_key.clone().into();
        let wallet = EthereumWallet::from(signer);
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .on_http(anvil_url.clone());
        let contract = Coprocessor::deploy(&provider).await.unwrap();
        Self {
            user: user_addr,
            contract: contract.address().clone(),
            provider,
            anvil_url,
            _anvil_instance: anvil_instance,
        }
    }

    pub async fn update_coprocessor(&self, canister_evm_address: Address) {
        Coprocessor::new(self.contract, &self.provider)
            .updateCoprocessor(canister_evm_address)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();
    }

    pub async fn new_job(&self, payment: U256) {
        Coprocessor::new(self.contract, &self.provider)
            .newJob()
            .value(payment)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();
    }

    pub async fn get_result(&self, id: U256) -> String {
        Coprocessor::new(self.contract, &self.provider)
            .getResult(id)
            .call()
            .await
            .unwrap()
            ._0
    }

    pub async fn transfer_eth(&self, addr: Address, amount: &str) {
        let tx = TransactionRequest::default()
            .with_to(addr)
            .with_value(parse_ether(amount).unwrap());
        self.provider
            .send_transaction(tx)
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();
    }

    pub async fn get_balance(&self, addr: Address) -> U256 {
        self.provider.get_balance(addr).await.unwrap()
    }

    #[allow(dead_code)]
    pub async fn mine_block(&self) {
        let response: serde_json::Value = self
            .provider
            .client()
            .request("evm_mine", json!({}))
            .await
            .unwrap();
        assert_eq!(response, "0x0");
    }
}

sol!(
    #[sol(rpc)]
    Coprocessor,
    "../out/Coprocessor.sol/Coprocessor.json"
);
