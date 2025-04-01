// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports, non_snake_case)]
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
    pub caller: super::Caller,
}

impl ChainFusionCanister {
    pub fn get_evm_address(&self) -> super::CallBuilder<Option<String>> {
        let args = Encode!();
        self.caller.call(
            self.canister_id,
            super::CallMode::Query,
            "get_evm_address",
            args,
        )
    }
}

pub fn new(caller: &super::Caller, canister_id: Principal) -> ChainFusionCanister {
    ChainFusionCanister {
        canister_id,
        caller: caller.clone(),
    }
}

pub fn deploy(
    deployer: &super::Deployer,
    arg0: InitArg,
) -> super::DeployBuilder<ChainFusionCanister> {
    let args = Encode!(&arg0);
    deployer.deploy(args, new)
}
