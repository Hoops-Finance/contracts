#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

#[contract]
pub struct Deployer;

#[contractimpl]
impl Deployer {
    pub fn deploy_account(
        env: Env,
        owner: Address,
        router: Address, // keep for future use
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
    ) -> Address {
        owner.require_auth();
        let _ = router; // suppress unused variable warning for now
        let acct_id = env.deployer().with_address(owner.clone(), salt).deploy_v2(wasm_hash, ());
        // router is currently unused, but kept for interface compatibility
        acct_id
    }
}
