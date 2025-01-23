//! This module contains functions for interacting with Ethereum contracts using JSON-RPC requests.
use ethers_core::abi::Token;
use ethers_core::types::U256;
use ethers_core::utils::hex;
use hex::FromHexError;
use serde::{Deserialize, Serialize};

use evm_rpc_canister_types::{
    CallArgs, CallResult, EvmRpcCanister, MultiCallResult, RpcServices, TransactionRequest,
};

use crate::eth_send_raw_transaction::{get_data, get_function, ContractDetails};

/// Represents the parameters for an Ethereum call.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EthCallParams {
    pub to: String,
    pub data: String,
}

/// Represents a JSON-RPC request for an Ethereum call.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EthCallJsonRpcRequest {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    pub params: (EthCallParams, String),
}

/// Retrieves the balance of an ERC20 token for a given account.
///
/// # Arguments
///
/// * `contract_address` - The address of the ERC20 token contract.
/// * `account` - The account to retrieve the balance for.
/// * `rpc_service` - The RPC service to use for the call.
/// * `evm_rpc` - The EVM RPC canister.
///
/// # Returns
///
/// The balance of the ERC20 token for the given account.
pub async fn erc20_balance_of(
    contract_address: String,
    account: String,
    rpc_services: RpcServices,
    evm_rpc: EvmRpcCanister,
) -> U256 {
    let cycles = 10_000_000_000;
    // Define the ABI JSON as a string literal
    let abi_json = r#"
   [
       {
           "constant": true,
           "inputs": [
               {
                   "name": "_owner",
                   "type": "address"
               }
           ],
           "name": "balanceOf",
           "outputs": [
               {
                   "name": "balance",
                   "type": "uint256"
               }
           ],
           "type": "function"
       }
   ]
   "#;
    let abi =
        serde_json::from_str::<ethers_core::abi::Contract>(abi_json).expect("should serialise");

    let contract_details = ContractDetails {
        contract_address,
        abi: &abi,
        function_name: "balanceOf",
        args: &[Token::Address(
            account.parse().expect("address should be valid"),
        )],
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

    let balance = match evm_rpc
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
                if let Token::Uint(value) = token {
                    value
                } else {
                    panic!("Expected Uint token")
                }
            }
            CallResult::Err(err) => panic!("Response error: {err:?}"),
        },
        MultiCallResult::Inconsistent(_) => panic!("Status is inconsistent"),
    };
    balance
}

/// Converts a byte slice to a hexadecimal string representation.
///
/// # Arguments
///
/// * `data` - The byte slice to convert.
///
/// # Returns
///
/// The hexadecimal string representation of the byte slice.
pub fn to_hex(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

/// Converts a hexadecimal string representation to a byte slice.
///
/// # Arguments
///
/// * `data` - The hexadecimal string to convert.
///
/// # Returns
///
/// The byte slice representation of the hexadecimal string, or an error if the conversion fails.
pub fn from_hex(data: &str) -> Result<Vec<u8>, FromHexError> {
    hex::decode(&data[2..])
}
