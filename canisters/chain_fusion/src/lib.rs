mod guard;
mod job;
mod lifecycle;
mod logs;
mod state;
// uncomment to enable serving stored assets via http requests
// mod storage;

use std::time::Duration;

use alloy::{network::TxSigner, signers::icp::IcpSigner, sol};
use logs::scrape_eth_logs;

use lifecycle::InitArg;
use state::{read_state, State};

use crate::state::{initialize_state, mutate_state};

pub const SCRAPING_LOGS_INTERVAL: Duration = Duration::from_secs(60);

sol!(
    #[sol(rpc)]
    "../../contracts/Coprocessor.sol"
);

fn setup_timers() {
    let ecdsa_key_name = read_state(State::key_id).name.clone();
    ic_cdk_timers::set_timer(Duration::ZERO, || {
        ic_cdk::spawn(async move {
            let signer = IcpSigner::new(vec![], &ecdsa_key_name, None).await.unwrap();
            let address = signer.address();
            mutate_state(|s| {
                s.signer = Some(signer);
                s.canister_evm_address = Some(address);
            });
        })
    });
    // // Start scraping logs almost immediately after the install, then repeat with the interval.
    ic_cdk_timers::set_timer(Duration::from_secs(10), || ic_cdk::spawn(scrape_eth_logs()));
}

#[ic_cdk::init]
fn init(arg: InitArg) {
    initialize_state(state::State::try_from(arg).expect("BUG: failed to initialize canister"));
    setup_timers();
}

#[ic_cdk::query]
fn get_evm_address() -> String {
    read_state(|s| s.canister_evm_address)
        .expect("evm address should be initialized")
        .to_string()
}

// uncomment this if you need to serve stored assets from `storage.rs` via http requests

// #[ic_cdk::query]
// fn http_request(req: HttpRequest) -> HttpResponse {
//     if let Some(asset) = get_asset(&req.path().to_string()) {
//         let mut response_builder = HttpResponseBuilder::ok();

//         for (name, value) in asset.headers {
//             response_builder = response_builder.header(name, value);
//         }

//         response_builder
//             .with_body_and_content_length(asset.body)
//             .build()
//     } else {
//         HttpResponseBuilder::not_found().build()
//     }
// }

// Enables Candid export, read more [here](https://internetcomputer.org/docs/current/developer-docs/backend/rust/generating-candid/)
ic_cdk::export_candid!();
