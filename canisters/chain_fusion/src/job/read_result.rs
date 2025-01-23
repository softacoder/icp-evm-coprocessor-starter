use ethers_core::abi::Token;
use evm_rpc_canister_types::{CallArgs, CallResult, MultiCallResult, TransactionRequest, EVM_RPC};
use ic_evm_utils::{
    eth_call::{from_hex, to_hex},
    eth_send_raw_transaction::{get_data, get_function, ContractDetails},
};

use crate::state::{read_state, State};

pub async fn read_result(index: String) -> String {
    let contract_address = read_state(State::get_logs_addresses)[0].clone();
    let rpc_services = read_state(State::rpc_services);
    let cycles = 10_000_000_000;
    // Define the ABI JSON as a string literal
    let abi_json = r#"
 [
{
            "type": "function",
            "name": "getResult",
            "inputs": [
                {
                    "name": "_job_id",
                    "type": "uint256",
                    "internalType": "uint256"
                }
            ],
            "outputs": [
                { "name": "", "type": "string", "internalType": "string" }
            ],
            "stateMutability": "view"
        }
 ]
 "#;
    let abi =
        serde_json::from_str::<ethers_core::abi::Contract>(abi_json).expect("should serialise");

    let contract_details = ContractDetails {
        contract_address,
        abi: &abi,
        function_name: "getResult",
        args: &[Token::Uint(index.parse().expect("index should be valid"))],
    };
    let function = get_function(&contract_details);
    let data = get_data(function, &contract_details);
    let transaction_request: TransactionRequest = TransactionRequest {
        to: Some(contract_details.contract_address.clone()),
        input: Some(to_hex(&data)),
        ..Default::default()
    };

    let call_args: CallArgs = CallArgs {
        transaction: transaction_request,
        block: None,
    };

    let result = match EVM_RPC
        .eth_call(rpc_services, None, call_args, cycles)
        .await
        .expect("Call failed")
        .0
    {
        MultiCallResult::Consistent(r) => match r {
            CallResult::Ok(ok) => {
                // this already returns the hex encoded response
                let result = from_hex(&ok).unwrap();
                let token = function
                    .decode_output(&result)
                    .expect("Error decoding output")
                    .first()
                    .unwrap()
                    .clone();
                if let Token::String(value) = token {
                    value
                } else {
                    panic!("Expected Uint token")
                }
            }
            CallResult::Err(err) => panic!("Response error: {err:?}"),
        },
        MultiCallResult::Inconsistent(_) => panic!("Status is inconsistent"),
    };
    result
}
