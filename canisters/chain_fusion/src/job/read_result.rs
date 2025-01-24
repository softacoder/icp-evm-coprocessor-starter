use alloy::{primitives::Uint, providers::ProviderBuilder, transports::icp::IcpConfig};
use ic_cdk::println;

use crate::{
    state::{read_state, State},
    Coprocessor,
};

pub async fn read_result(job_id: Uint<256, 4>) {
    let rpc_service = read_state(|s| s.rpc_service.clone());
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    let contract_address = read_state(State::get_filter_addresses)[0];
    let contract = Coprocessor::new(contract_address, provider);

    let response = contract.getResult(job_id).call().await;

    let result = match response {
        Ok(result) => result._0,
        Err(e) => panic!("{}", e.to_string()),
    };
    println!("Result: {}", result);
}
