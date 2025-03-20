#![cfg(test)]


use alloy::{hex::FromHex, primitives::Address};
use candid::{CandidType, Encode, Principal};
use evm_rpc_canister_types::{RpcApi, RpcService};
use helpers::{
    evm::EvmEnv,
    http_outcalls::handle_http_outcalls,
    icp::{query, update, Canister, EmptyRecord},
};
use ic_cdk::api::management_canister::{ecdsa::EcdsaKeyId, main::CanisterId};
use lazy_static::lazy_static;
use pocket_ic::{nonblocking::PocketIc, PocketIcBuilder};
use serde::Deserialize;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{sync::Mutex, task};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    pub rpc_service: RpcService,
    pub chain_id: u64,
    pub filter_addresses: Vec<String>,
    pub coprocessor_evm_address: String,
    pub filter_events: Vec<String>,
    pub ecdsa_key_id: EcdsaKeyId,
}

mod helpers;
mod tests;

lazy_static! {
    static ref WORKSPACE_ROOT: PathBuf = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("Failed to get workspace root")
        .workspace_root
        .into();
}

struct TestEnv {
    pic: Arc<Mutex<PocketIc>>,
    user: Principal,
    chain_fusion: CanisterId,
    evm: EvmEnv,
}

impl TestEnv {
    async fn new() -> Self {
        std::env::set_var("RUST_LOG", "error");

        let evm = EvmEnv::new().await;

        let pic = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_ii_subnet()
            .build_async()
            .await;

        pic.set_time(
            SystemTime::UNIX_EPOCH
                .checked_add(Duration::from_secs(1738933200))
                .unwrap(),
        )
        .await;

        let controller =
            Principal::from_text("6vui3-u5w5r-6ks6a-ojcem-giomu-gmfhx-bjohd-kpkos-3xlmt-n7bn7-wae")
                .unwrap();

        let user =
            Principal::from_text("y3flu-q4efd-gss2z-iz4vu-eyroy-blzhh-4f27c-y5fsx-h7ckx-un4vy-4ae")
                .unwrap();

        let chain_fusion = pic
            .create_canister_with_settings(Some(controller), None)
            .await;
        let rpc_node_url = "http://localhost:8545".to_string();
        pic.add_cycles(chain_fusion, u64::MAX.into()).await;
        pic.install_canister(
            chain_fusion,
            Canister::ChainFusion.wasm(),
            Encode!(&InitArg {
                rpc_service: RpcService::Custom(RpcApi {
                    url: rpc_node_url.clone(),
                    headers: None,
                }),
                chain_id: 31337,
                filter_addresses: vec![evm.contract.to_string(),],
                coprocessor_evm_address: evm.contract.to_string(),
                filter_events: vec!["NewJob(uint256)".to_string()],
                ecdsa_key_id: EcdsaKeyId {
                    curve: ic_cdk::api::management_canister::ecdsa::EcdsaCurve::Secp256k1,
                    name: "dfx_test_key".to_string(),
                }
            })
            .unwrap(),
            Some(controller),
        )
        .await;

        let evm_rpc = pic
            .create_canister_with_id(Some(controller), None, Canister::EvmRpc.id())
            .await
            .unwrap();
        pic.add_cycles(evm_rpc, u64::MAX.into()).await;
        pic.install_canister(
            evm_rpc,
            Canister::EvmRpc.wasm(),
            Encode!(&EmptyRecord {}).unwrap(),
            Some(controller),
        )
        .await;

        let test = TestEnv {
            pic: Arc::new(Mutex::new(pic)),
            user,
            chain_fusion,
            evm,
        };

        while test.get_evm_address().await.is_none() {
            test.tick().await;
        }

        let canister_evm_address =
            Address::from_hex(test.get_evm_address().await.unwrap()).unwrap();

        test.evm.update_coprocessor(canister_evm_address).await;

        test.evm.transfer_eth(canister_evm_address, "1").await;

        let pic = Arc::downgrade(&test.pic);
        task::spawn(handle_http_outcalls(
            pic,
            test.evm.anvil_url.clone(),
            vec![rpc_node_url],
        ));
        test
    }

    pub async fn tick(&self) {
        let pic = self.pic.lock().await;
        pic.advance_time(Duration::from_secs(1)).await;
        pic.tick().await;
    }

    #[allow(dead_code)]
    async fn update<T>(
        &self,
        canister: CanisterId,
        caller: Principal,
        method: &str,
        arg: impl CandidType,
    ) -> Result<T, String>
    where
        T: for<'a> Deserialize<'a> + CandidType,
    {
        let pic = self.pic.lock().await;
        update(&pic, canister, caller, method, arg).await
    }

    async fn query<T>(
        &self,
        canister: CanisterId,
        caller: Principal,
        method: &str,
        arg: impl CandidType,
    ) -> Result<T, String>
    where
        T: for<'a> Deserialize<'a> + CandidType,
    {
        let pic = self.pic.lock().await;
        query(&pic, canister, caller, method, arg).await
    }

    pub async fn get_evm_address(&self) -> Option<String> {
        self.query::<Option<String>>(self.chain_fusion, self.user, "get_evm_address", ())
            .await
            .unwrap()
    }
}
