#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol, Val, Vec};

#[contract]
pub struct Deployer;

#[contractimpl]
impl Deployer {
    pub fn deploy_account(
        env: Env,
        owner: Address,
        router: Address,
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
    ) -> Address {
        owner.require_auth();
        let acct_id =
            env.deployer().with_address(owner.clone(), salt).deploy(wasm_hash);
        let _: Val = env.invoke_contract(
            &acct_id,
            &Symbol::short("initialize"),
            Vec::from_array(&env, [owner.into(), router.into()]),
        );
        acct_id
    }
}
