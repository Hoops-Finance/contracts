#![no_std]

mod client;
mod storage;
mod types;

use soroban_sdk::{contract, contracterror, contractimpl, panic_with_error, token, Address, Env, IntoVal, Symbol, Val, Vec};

use crate::storage::{
    get_adapters, get_core_config, set_adapters, set_core_config,
    get_pool, set_pool, get_pools_for_pair, add_pool_to_pair_index,
};
use crate::types::{CoreConfig, LpPlan, MarketData, RedeemPlan};
use hoops_adapter_interface::AdapterClient;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RouterError {
    AlreadyInitialized = 10,
    InvalidID = 11,
    DefaultError = 100,
    UnsupportedPair = 101,
    ExternalFailure = 102,
    PoolNotFound = 200,
    InsufficientLpBalance = 201,
    MinAmountNotMet = 202,
    MaxInRatio = 203,
    MaxOutRatio = 204,
    DeadlinePassed = 205,
    NotInitialized = 206,
    InvalidArgument = 207,
    MultipathUnsupported = 208,
    InvalidAmount = 209,
    InvalidPath = 210,
    InsufficientBalance = 211,
    AdapterNotFound = 212,
    Expired = 213,
    PoolValidationFailed = 214,
}

/// Resolve a pool with full token context: return MarketData from persistent
/// storage, or lazy-validate via the adapter and register the pool.
fn resolve_pool_with_tokens(
    e: &Env,
    pool_address: &Address,
    adapter_id: i128,
    token_a: &Address,
    token_b: &Address,
) -> MarketData {
    // Fast path: pool already registered
    if let Some(market) = get_pool(e, pool_address) {
        return market;
    }

    // Slow path: lazy validation via adapter
    let adapters = get_adapters(e);
    let Some(adapter_address) = adapters.get(adapter_id) else {
        panic_with_error!(e, RouterError::AdapterNotFound);
    };

    let adapter = AdapterClient::new(e, &adapter_address);
    let result = adapter.try_validate_pool(pool_address, token_a, token_b);

    match result {
        Ok(Ok(pool_info)) => {
            let market = MarketData {
                adapter_id,
                lp_token: pool_info.lp_token,
                pool_address: pool_info.pool_address.clone(),
                pool_type: pool_info.pool_type,
                token_a: pool_info.token_a.clone(),
                token_b: pool_info.token_b.clone(),
            };
            set_pool(e, &pool_info.pool_address, &market);
            add_pool_to_pair_index(e, &pool_info.token_a, &pool_info.token_b, &pool_info.pool_address);
            market
        }
        _ => {
            panic_with_error!(e, RouterError::PoolValidationFailed);
        }
    }
}

pub trait HoopsRouterTrait {
    fn initialize(e: Env, admin: Address);
    fn get_version(e: Env) -> u32;
    fn add_adapter(e: Env, adapter_id: i128, adapter_address: Address);
    fn remove_adapter(e: Env, adapter_id: i128);
    fn add_markets(e: Env, markets: Vec<MarketData>);
    fn get_all_quotes(
        e: Env,
        amount: i128,
        token_in: Address,
        token_out: Address,
    ) -> Vec<crate::types::SwapQuote>;
    fn get_best_quote(
        e: Env,
        amount: i128,
        token_in: Address,
        token_out: Address,
    ) -> Option<crate::types::SwapQuote>;
    fn swap(e: Env, amount: i128, token_in: Address, token_out: Address, best_hop: Address, sender: Address, deadline: u64);
    fn provide_liquidity(
        e: Env,
        amount: i128,
        lp_plans: Vec<LpPlan>,
        sender: Address,
        deadline: u64,
    );
    fn batch_redeem_liquidity(
        e: Env,
        plans: Vec<RedeemPlan>,
        sender: Address,
        deadline: u64,
    );
}

#[contract]
pub struct HoopsRouter;

#[contractimpl]
impl HoopsRouterTrait for HoopsRouter {
    fn initialize(e: Env, admin: Address) {
        let config = CoreConfig { admin, version: 2 };
        set_core_config(&e, &config);
    }

    fn get_version(e: Env) -> u32 {
        get_core_config(&e).version
    }

    fn add_adapter(e: Env, adapter_id: i128, adapter_address: Address) {
        let config = get_core_config(&e);
        config.admin.require_auth();

        let mut adapters = get_adapters(&e);
        adapters.set(adapter_id, adapter_address);
        set_adapters(&e, &adapters);
    }

    fn remove_adapter(e: Env, adapter_id: i128) {
        let config = get_core_config(&e);
        config.admin.require_auth();

        let mut adapters = get_adapters(&e);
        adapters.remove(adapter_id);
        set_adapters(&e, &adapters);
    }

    /// Admin fallback: register pools directly. Writes to persistent storage + pair index.
    fn add_markets(e: Env, markets_to_add: Vec<MarketData>) {
        let config = get_core_config(&e);
        config.admin.require_auth();

        for market in markets_to_add.iter() {
            set_pool(&e, &market.pool_address, &market);
            add_pool_to_pair_index(&e, &market.token_a, &market.token_b, &market.pool_address);
        }
    }

    fn get_all_quotes(
        e: Env,
        amount: i128,
        token_in: Address,
        token_out: Address,
    ) -> Vec<crate::types::SwapQuote> {
        let adapters = get_adapters(&e);
        let mut quotes = Vec::new(&e);

        // Use token-pair index for O(1) pool lookup
        let pool_addresses = get_pools_for_pair(&e, &token_in, &token_out);

        for pool_addr in pool_addresses.iter() {
            let Some(market) = get_pool(&e, &pool_addr) else { continue; };
            let Some(adapter_address) = adapters.get(market.adapter_id) else { continue; };
            let adapter = AdapterClient::new(&e, &adapter_address);
            if let Ok(amount_out) = adapter.try_quote_in(&market.pool_address, &amount, &token_in, &token_out) {
                if amount_out.is_ok() {
                    quotes.push_back(crate::types::SwapQuote {
                        adapter_id: market.adapter_id,
                        pool_address: market.pool_address.clone(),
                        token_in: token_in.clone(),
                        token_out: token_out.clone(),
                        amount_in: amount,
                        amount_out: amount_out.unwrap(),
                        pool_type: market.pool_type,
                        lp_token: market.lp_token.clone(),
                    });
                }
            }
        }
        quotes
    }

    fn get_best_quote(
        e: Env,
        amount: i128,
        token_in: Address,
        token_out: Address,
    ) -> Option<crate::types::SwapQuote> {
        let all_quotes =
            Self::get_all_quotes(e.clone(), amount, token_in.clone(), token_out.clone());
        let mut best: Option<crate::types::SwapQuote> = None;
        for quote in all_quotes.iter() {
            if best.is_none() || quote.amount_out > best.as_ref().unwrap().amount_out {
                best = Some(quote);
            }
        }
        best
    }

    fn swap(e: Env, amount: i128, token_in: Address, token_out: Address, best_hop: Address, sender: Address, _deadline: u64) {
        // Tokens are already in the Router (transferred by the Account contract).
        let adapters = get_adapters(&e);

        // O(1) lookup by pool address
        let Some(market) = get_pool(&e, &best_hop) else {
            panic_with_error!(&e, RouterError::PoolNotFound);
        };

        let Some(adapter_address) = adapters.get(market.adapter_id) else {
            panic_with_error!(&e, RouterError::AdapterNotFound);
        };

        // Transfer tokens from Router to the Adapter.
        token::Client::new(&e, &token_in)
            .transfer(&e.current_contract_address(), &adapter_address, &amount);

        // Delegate to the adapter's swap_in_pool function.
        let mut args: Vec<Val> = Vec::new(&e);
        args.push_back(amount.into_val(&e));
        args.push_back(0i128.into_val(&e));
        args.push_back(token_in.into_val(&e));
        args.push_back(token_out.into_val(&e));
        args.push_back(best_hop.into_val(&e));
        args.push_back(sender.into_val(&e));

        let _: i128 = e.invoke_contract(
            &adapter_address,
            &Symbol::new(&e, "swap_in_pool"),
            args,
        );
    }

    fn provide_liquidity(
        e: Env,
        _amount: i128,
        lp_plans: Vec<LpPlan>,
        sender: Address,
        _deadline: u64,
    ) {
        // Tokens are already in the Router (transferred by the Account contract).
        let adapters = get_adapters(&e);

        for plan in lp_plans.iter() {
            let Some(adapter_address) = adapters.get(plan.adapter_id) else { continue; };

            // Lazy validate / resolve the pool (O(1) if already known)
            let _market = resolve_pool_with_tokens(
                &e,
                &plan.pool_address,
                plan.adapter_id,
                &plan.token_a,
                &plan.token_b,
            );

            // Transfer both tokens from Router to Adapter
            token::Client::new(&e, &plan.token_a)
                .transfer(&e.current_contract_address(), &adapter_address, &plan.amount_a);
            token::Client::new(&e, &plan.token_b)
                .transfer(&e.current_contract_address(), &adapter_address, &plan.amount_b);

            // Delegate to adapter's add_liq_in_pool
            let mut args: Vec<Val> = Vec::new(&e);
            args.push_back(plan.token_a.into_val(&e));
            args.push_back(plan.token_b.into_val(&e));
            args.push_back(plan.amount_a.into_val(&e));
            args.push_back(plan.amount_b.into_val(&e));
            args.push_back(plan.pool_address.into_val(&e));
            args.push_back(sender.into_val(&e));

            let _: i128 = e.invoke_contract(
                &adapter_address,
                &Symbol::new(&e, "add_liq_in_pool"),
                args,
            );
        }
    }

    fn batch_redeem_liquidity(
        e: Env,
        plans: Vec<RedeemPlan>,
        sender: Address,
        deadline: u64,
    ) {
        let timestamp = e.ledger().timestamp();
        if timestamp > deadline {
            panic_with_error!(&e, RouterError::Expired);
        }

        let adapters = get_adapters(&e);

        for plan in plans.iter() {
            let Some(adapter_address) = adapters.get(plan.adapter_id) else {
                panic_with_error!(&e, RouterError::AdapterNotFound);
            };

            // Use pool_address directly from the plan — O(1), no Vec scan
            let pool = plan.pool_address.clone();

            // Transfer LP from Router to Adapter.
            token::Client::new(&e, &plan.lp_token)
                .transfer(&e.current_contract_address(), &adapter_address, &plan.lp_amount);

            // Call adapter's remove_liq_from_pool
            let mut args: Vec<Val> = Vec::new(&e);
            args.push_back(pool.into_val(&e));
            args.push_back(plan.lp_amount.into_val(&e));
            args.push_back(sender.into_val(&e));
            let _: (i128, i128) = e.invoke_contract(
                &adapter_address,
                &Symbol::new(&e, "remove_liq_from_pool"),
                args,
            );
        }
    }
}
