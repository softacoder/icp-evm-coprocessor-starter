// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use crate::DeployBuilder;

use super::{CallBuilder, CallMode, Provider};
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};

#[derive(CandidType, Deserialize)]
pub enum EcdsaCurve {
    #[serde(rename = "secp256k1")]
    Secp256K1,
}

#[derive(CandidType, Deserialize)]
pub struct EcdsaKeyId {
    pub name: String,
    pub curve: EcdsaCurve,
}

#[derive(CandidType, Deserialize)]
pub enum L2MainnetService {
    Alchemy,
    BlockPi,
    PublicNode,
    Ankr,
}

#[derive(CandidType, Deserialize)]
pub struct HttpHeader {
    pub value: String,
    pub name: String,
}

#[derive(CandidType, Deserialize)]
pub struct RpcApi {
    pub url: String,
    pub headers: Option<Vec<HttpHeader>>,
}

#[derive(CandidType, Deserialize)]
pub enum EthMainnetService {
    Alchemy,
    BlockPi,
    Cloudflare,
    PublicNode,
    Ankr,
}

#[derive(CandidType, Deserialize)]
pub enum RpcService {
    EthSepolia(L2MainnetService),
    BaseMainnet(L2MainnetService),
    Custom(RpcApi),
    OptimismMainnet(L2MainnetService),
    ArbitrumOne(L2MainnetService),
    EthMainnet(EthMainnetService),
    Chain(u64),
    Provider(u64),
}

#[derive(CandidType, Deserialize)]
pub struct InitArg {
    pub ecdsa_key_id: EcdsaKeyId,
    pub rpc_service: RpcService,
    pub filter_addresses: Vec<String>,
    pub chain_id: u64,
    pub coprocessor_evm_address: String,
    pub filter_events: Vec<String>,
}

pub struct ChainFusionCanister {
    pub canister_id: Principal,
    pub provider: Provider,
}

impl ChainFusionCanister {
    pub fn get_evm_address(&self) -> CallBuilder<Option<String>> {
        let args = Encode!(&());
        self.provider
            .call(self.canister_id, CallMode::Query, "get_evm_address", args)
    }
}

pub fn new(provider: &Provider, canister_id: Principal) -> ChainFusionCanister {
    ChainFusionCanister {
        canister_id,
        provider: provider.clone(),
    }
}

pub fn deploy(provider: &Provider, arg: InitArg) -> DeployBuilder<ChainFusionCanister> {
    let args = Encode!(&arg);
    let provider_clone = provider.clone();
    provider.deploy(
        args,
        Box::new(move |canister_id| new(&provider_clone, canister_id)),
    )
}
