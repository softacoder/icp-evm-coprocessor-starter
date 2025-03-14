use std::str::FromStr;

use crate::TestEnv;
use alloy::primitives::{utils::parse_ether, Address, U256};

#[tokio::test]
async fn test_coprocessor_job() {
    let test = TestEnv::new().await;
    let canister_evm_addr = Address::from_str(&test.get_evm_address().await.unwrap()).unwrap();

    let user_balance_before = test.evm.get_balance(test.evm.user).await;
    let canister_balance_before = test.evm.get_balance(canister_evm_addr).await;
    let payment = parse_ether("0.1").unwrap();
    test.evm.new_job(payment).await;

    let user_balance_after = test.evm.get_balance(test.evm.user).await;
    let canister_balance_after = test.evm.get_balance(canister_evm_addr).await;
    assert_eq!(canister_balance_before + payment, canister_balance_after);
    // This is not a strict equality because of gas cost payments.
    assert!(user_balance_before - payment >= user_balance_after);

    for _ in 0..100 {
        test.tick().await;
    }

    let result = test.evm.get_result(U256::from(0)).await;
    // The 20-th fibonacci number.

    assert_eq!(result, "6765");
}
