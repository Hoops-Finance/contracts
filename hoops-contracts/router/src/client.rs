use soroban_sdk::{contractclient, Address, BytesN, Env, Vec};
use crate::{RouterError, LpPlan};

//#[contractclient(name = "RouterClient")]
pub trait RouterTrait {
    fn initialize(e: Env, admin_addr: Address) -> Result<(), RouterError>;
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), RouterError>;
    fn add_adapter(e: Env, id: i128, adapter: Address) -> Result<(), RouterError>;
    fn remove_adapter(e: Env, id: i128) -> Result<(), RouterError>;
    fn swap_exact_in(
        e: Env,
        adapter_id: i128,
        amount_in: i128,
        min_out: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<i128, RouterError>;
    fn provide_liquidity(
        e: Env, 
        usdc_amt: i128, 
        plans: Vec<LpPlan>,
        beneficiary: Address, 
        deadline: u64,
    ) -> Result<(), RouterError>;
    fn redeem_liquidity(
        e: Env, 
        lp_token: Address, 
        lp_amt: i128,
        beneficiary: Address, 
        deadline: u64,
    ) -> Result<(), RouterError>;
    fn admin(e: &Env) -> Address;
}
