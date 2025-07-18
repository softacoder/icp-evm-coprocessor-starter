mod calculate_result;
mod read_result;
mod submit_result;

use alloy::rpc::types::Log;
use read_result::read_result;
use submit_result::submit_result;

use crate::{
    job::calculate_result::fibonacci,
    state::{mutate_state, LogSource},
    Coprocessor,
};
// here
pub async fn job(log_source: LogSource, log: Log) {
    mutate_state(|s| s.record_processed_log(log_source.clone()));
    // because we deploy the canister with topics only matching
    // NewJob events we can safely assume that the event is a NewJob.
    let new_job: Log<Coprocessor::NewJob> = log.log_decode().unwrap();
    let Coprocessor::NewJob { job_id } = new_job.data();
    // this calculation would likely exceed an ethereum blocks gas limit
    // but can easily be calculated on the IC
    let result = fibonacci(20);
    // we write the result back to the evm smart contract, creating a signature
    // on the transaction with chain key ecdsa and sending it to the evm via the
    // evm rpc canister
    submit_result(result.to_string(), *job_id).await;
    // `read_result` demonstrates how to make a `eth_call` via the evm rpc canister
    read_result(*job_id).await;
}
