use std::path::PathBuf;

use candid::{decode_one, encode_one, CandidType, Principal};
use ic_cdk::api::call::reply;
use pocket_ic::{management_canister::CanisterId, nonblocking::PocketIc, RejectResponse};
use serde::Deserialize;

use crate::WORKSPACE_ROOT;

pub const CANISTERS: [(Canister, Location); 2] = [
    (
        Canister::ChainFusion,
        Location::Local(("chain_fusion", Some("2222s-4iaaa-aaaaf-ax2uq-cai"))),
    ),
    (
        Canister::EvmRpc,
        Location::Local(("evm_rpc", Some("7hfb6-caaaa-aaaar-qadga-cai"))),
    ),
];

#[derive(CandidType)]
pub struct EmptyRecord {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Canister {
    ChainFusion,
    EvmRpc,
}

impl Canister {
    pub fn location(&self) -> Location {
        CANISTERS.iter().find(|(c, _)| c == self).unwrap().1
    }

    pub fn id(&self) -> CanisterId {
        match self.location() {
            Location::Local((_, id)) => CanisterId::from_text(id.unwrap()).unwrap(),
            Location::Pulled(id) => CanisterId::from_text(id).unwrap(),
        }
    }

    pub fn wasm(&self) -> Vec<u8> {
        let location = self.location();
        let mut path = match location {
            Location::Local((name, _)) => {
                let mut path = WORKSPACE_ROOT.clone();
                path.push(".dfx");
                path.push("local");
                path.push("canisters");
                path.push(name);
                path.push(format!("{}.wasm", name));
                path
            }
            Location::Pulled(id) => {
                let mut path = PathBuf::new();
                path.push(std::env::var("HOME").unwrap());
                path.push(".cache");
                path.push("dfinity");
                path.push("pulled");
                path.push(id);
                path.push("canister.wasm.gz");
                path
            }
        };

        if !path.exists() && Some("wasm") == path.extension().map(|x| x.to_str().unwrap()) {
            path.set_extension("wasm.gz");
        }

        std::fs::read(path.as_path())
            .unwrap_or_else(|_| panic!("wasm binary not found: {:?}", path))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Location {
    Local((&'static str, Option<&'static str>)),
    #[allow(dead_code)]
    Pulled(&'static str),
}

pub async fn update<T>(
    pic: &PocketIc,
    canister: CanisterId,
    caller: Principal,
    method: &str,
    arg: impl CandidType,
) -> Result<T, String>
where
    T: for<'a> Deserialize<'a> + CandidType,
{
    let result: Result<Vec<u8>, RejectResponse> = pic
        .update_call(canister, caller, method, encode_one(arg).unwrap())
        .await;

    match result {
        Ok(reply) => decode_one(&reply).unwrap(),
        Err(rr) => Err(rr.reject_message),
    }
}

pub async fn query<T>(
    pic: &PocketIc,
    canister: CanisterId,
    caller: Principal,
    method: &str,
    arg: impl CandidType,
) -> Result<T, String>
where
    T: for<'a> Deserialize<'a> + CandidType,
{
    let result = pic
        .query_call(canister, caller, method, encode_one(arg).unwrap())
        .await;

    match result {
        Ok(reply) => Ok(decode_one(&reply).unwrap()),
        Err(rr) => Err(rr.reject_message),
    }
}
