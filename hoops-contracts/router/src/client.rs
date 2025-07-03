use crate::types::{LpPlan, MarketData, SwapQuote};
use crate::RouterError;
use soroban_sdk::{contractclient, Address, BytesN, Env, Vec};

#[contractclient(name = "RouterClient")]
pub trait RouterTrait {
    /* ---------- lifecycle ---------- */
    fn initialize(e: &Env, admin_addr: &Address) -> Result<(), RouterError>;
    fn upgrade(e: &Env, new_wasm_hash: &BytesN<32>) -> Result<(), RouterError>;

    /* ---------- admin ops ---------- */
    fn add_adapter(e: &Env, id: &i128, adapter: &Address) -> Result<(), RouterError>;
    fn remove_adapter(e: &Env, id: &i128) -> Result<(), RouterError>;
    fn add_markets(e: &Env, markets: &Vec<MarketData>) -> Result<(), RouterError>;
    fn admin(e: &Env) -> Address;

    /* ---------- swaps ---------- */
    fn swap_exact_in(
        e: &Env,
        adapter_id: &i128,
        amount_in: &i128,
        min_out: &i128,
        path: Vec<Address>,
        to: &Address,
        deadline: &u64,
    ) -> Result<i128, RouterError>;

    fn swap_exact_in_best(
        e: &Env,
        amount_in: &i128,
        min_out: &i128,
        path: Vec<Address>,
        to: &Address,
        deadline: &u64,
    ) -> Result<i128, RouterError>;

    fn find_best_swap(
        e: &Env,
        amount_in: &i128,
        token_in: &Address,
        token_out: &Address,
    ) -> Result<(i128, i128), RouterError>;

    /* ---------- liquidity (stubs) ---------- */
    fn provide_liquidity(
        e: &Env,
        usdc_amt: &i128,
        plans: Vec<LpPlan>,
        beneficiary: &Address,
        deadline: &u64,
    ) -> Result<(), RouterError>;

    fn redeem_liquidity(
        e: &Env,
        lp_token: &Address,
        lp_amt: &i128,
        beneficiary: &Address,
        deadline: &u64,
    ) -> Result<(), RouterError>;

    fn get_all_quotes(
        e: &Env,
        token_in: &Address,
        token_out: &Address,
        amount_in: &u128,
    ) -> Vec<SwapQuote>;

    fn get_best_quote(
        e: &Env,
        token_in: &Address,
        token_out: &Address,
        amount_in: &u128,
    ) -> Option<SwapQuote>;

    fn swap(
        e: &Env,
        token_in: &Address,
        token_out: &Address,
        amount_in: &u128,
        dest_address: &Address,
    );

    fn discover_soroswap_pools(e: &Env, factory: &Address);

    fn discover_aqua_pools(e: &Env, router: &Address);

    fn discover_phoenix_pools(e: &Env, factory: &Address);
    fn discover_comet_pools(e: &Env, factory: &Address);
}
