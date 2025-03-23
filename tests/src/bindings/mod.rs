#![allow(dead_code, unused_imports, non_snake_case)]

type CallMode = ic_test::CallMode;
type Caller = ic_test::IcUser;
type CallBuilder<R> = ic_test::CallBuilder<R, ic_test::IcUser>;
type DeployMode = ic_test::DeployMode;
type Deployer = ic_test::IcUser;
type DeployBuilder<C> = ic_test::DeployBuilder<C, Caller>;

// candid: https://github.com/internet-computer-protocol/evm-rpc-canister/releases/latest/download/evm_rpc.did
// generated from: .dfx/local/canisters/evm_rpc/constructor.did
pub mod evm_rpc;

// candid: canisters/chain_fusion/chain_fusion.did
// generated from: canisters/chain_fusion/chain_fusion.did
pub mod chain_fusion;

pub mod evm {
    use alloy::sol;

    sol!(
        #[sol(rpc)]
        Coprocessor,
        "../out/Coprocessor.sol/Coprocessor.json",
    );
}
