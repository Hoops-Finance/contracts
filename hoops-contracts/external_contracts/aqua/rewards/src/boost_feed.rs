mod boost_feed {
    soroban_sdk::contractimport!(
        file = "../../../bytecodes/aqua_locker_feed_contract.wasm"
    );
}
pub use boost_feed::Client as RewardBoostFeedClient;
