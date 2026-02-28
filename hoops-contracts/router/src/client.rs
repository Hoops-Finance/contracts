use crate::types::{LpPlan, MarketData, RedeemPlan, SwapQuote};
use soroban_sdk::{contractclient, Address, Env, Vec};

#[contractclient(name = "RouterClient")]
pub trait RouterTrait {
    /* ---------- lifecycle ---------- */
    fn initialize(e: &Env, admin: &Address);
    fn get_version(e: &Env) -> u32;

    /* ---------- admin ops ---------- */
    fn add_adapter(e: &Env, adapter_id: &i128, adapter_address: &Address);
    fn remove_adapter(e: &Env, adapter_id: &i128);
    fn add_markets(e: &Env, markets: &Vec<MarketData>);

    /* ---------- quotes ---------- */
    fn get_all_quotes(
        e: &Env,
        amount: &i128,
        token_in: &Address,
        token_out: &Address,
    ) -> Vec<SwapQuote>;

    fn get_best_quote(
        e: &Env,
        amount: &i128,
        token_in: &Address,
        token_out: &Address,
    ) -> Option<SwapQuote>;

    /* ---------- swaps ---------- */
    fn swap(
        e: &Env,
        amount: &i128,
        token_in: &Address,
        token_out: &Address,
        best_hop: &Address,
        sender: &Address,
        deadline: &u64,
    );

    /* ---------- liquidity ---------- */
    fn provide_liquidity(
        e: &Env,
        amount: &i128,
        lp_plans: &Vec<LpPlan>,
        sender: &Address,
        deadline: &u64,
    );

    fn batch_redeem_liquidity(
        e: &Env,
        plans: &Vec<RedeemPlan>,
        sender: &Address,
        deadline: &u64,
    );
}
