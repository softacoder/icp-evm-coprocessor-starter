use alloy::primitives::Uint;
use alloy::providers::Provider;
use alloy::{network::EthereumWallet, providers::ProviderBuilder, transports::icp::IcpConfig};
use ic_cdk::println;

use crate::state::{mutate_state, read_state};
use crate::Coprocessor;

pub async fn submit_result(result: String, job_id: Uint<256, 4>) {
    // get necessary global state
    let signer = read_state(|s| s.signer.clone()).unwrap();
    let evm_address = read_state(|s| s.canister_evm_address).unwrap();
    let wallet = EthereumWallet::new(signer);
    let rpc_service = read_state(|s| s.rpc_service.clone());
    let chain_id = read_state(|s| s.chain_id);
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);
    let contract_address = read_state(|s| s.coprocessor_evm_address);
    let contract = Coprocessor::new(contract_address, provider.clone());

    // Attempt to get nonce from thread-local storage
    let maybe_nonce = read_state(|s| {
        // If a nonce exists, the next nonce to use is latest nonce + 1
        s.nonce.map(|nonce| nonce + 1)
    });

    // If no nonce exists, get it from the provider
    let nonce = if let Some(nonce) = maybe_nonce {
        nonce
    } else {
        provider
            .get_transaction_count(evm_address)
            .await
            .unwrap_or(0)
    };

    match contract
        .callback(result, job_id)
        .nonce(nonce)
        .from(evm_address)
        .chain_id(chain_id)
        .send()
        .await
    {
        Ok(res) => {
            let node_hash = *res.tx_hash();
            let tx_response = contract
                .provider()
                .get_transaction_by_hash(node_hash)
                .await
                .unwrap();

            match tx_response {
                Some(_tx) => {
                    // The transaction has been mined and included in a block, the nonce
                    // has been consumed. Save it to thread-local storage. Next transaction
                    // for this address will use a nonce that is = this nonce + 1
                    mutate_state(|s| {
                        s.nonce = Some(nonce);
                    });
                    println!("Successfully ran job {}, tx: {}", job_id, res.tx_hash())
                }
                None => println!("{}", "Could not get transaction.".to_string()),
            }
        }
        Err(e) => {
            println!("{}", e.to_string())
        }
    }
}
