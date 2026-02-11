#![no_std]

mod client;
mod storage;
mod types;

use soroban_sdk::{contract, contracterror, contractimpl, panic_with_error, token, Address, Env, String, Vec};

use crate::storage::{
    get_adapters, get_core_config, get_markets, set_adapters, set_core_config, set_markets,
};
use crate::types::{CoreConfig, LpPlan, MarketData};
use hoops_adapter_interface::AdapterClient;
/*
pub mod adapter_interface {
    soroban_sdk::contractimport!(file = "../bytecodes/hoops_adapter_interface.wasm");
    pub type AdapterClient<'a> = Client<'a>;
}
pub use adapter_interface::AdapterClient;*/

pub mod soroswap_factory {
    soroban_sdk::contractimport!(file = "../bytecodes/soroswap_factory.wasm");
    pub type SoroswapClient<'a> = Client<'a>;
}
pub use soroswap_factory::SoroswapClient;
pub mod soroswap_pair {
    soroban_sdk::contractimport!(file = "../bytecodes/soroswap_pair.wasm");
    pub type SoroswapPairClient<'a> = Client<'a>;
}
pub use soroswap_pair::SoroswapPairClient;

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
    fn redeem_liquidity(
        e: Env,
        lp_token: Address,
        lp_amount: i128,
        sender: Address,
        deadline: u64,
    );

    // Pool discovery
    fn discover_soroswap_pools(e: Env, factory: Address, pairs_to_check: Vec<(Address, Address)>);
    fn discover_aqua_pools(e: Env, factory: Address, pairs_to_check: Vec<(Address, Address)>);
    fn discover_phoenix_pools(e: Env, factory: Address, pairs_to_check: Vec<(Address, Address)>);
    fn discover_comet_pools(e: Env, factory: Address, pairs_to_check: Vec<(Address, Address)>);
}

#[contract]
pub struct HoopsRouter;

#[contractimpl]
impl HoopsRouterTrait for HoopsRouter {
    fn initialize(e: Env, admin: Address) {
        let config = CoreConfig { admin, version: 1 };
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

    /// Add a new market to the router. Example usage:
    ///
    /// ```ignore
    /// router.add_markets(e, vec![MarketData {
    ///     adapter_id: 0, // protocol id
    ///     pool_address: ...,
    ///     lp_token: ...,
    ///     token_a: ...,
    ///     token_b: ...,
    ///     reserve_a: ...,
    ///     reserve_b: ...,
    ///     pool_type: 0,
    /// }]);
    /// ```
    fn add_markets(e: Env, markets_to_add: Vec<MarketData>) {
        let config = get_core_config(&e);
        config.admin.require_auth();

        let mut markets = get_markets(&e);
        for market in markets_to_add.iter() {
            markets.push_back(market);
        }
        set_markets(&e, &markets);
    }

    fn get_all_quotes(
        e: Env,
        amount: i128,
        token_in: Address,
        token_out: Address,
    ) -> Vec<crate::types::SwapQuote> {
        let markets = get_markets(&e);
        let adapters = get_adapters(&e);
        let mut quotes = Vec::new(&e);

        // Canonicalize token order
        let (token_a, token_b) = if token_in < token_out {
            (token_in.clone(), token_out.clone())
        } else {
            (token_out.clone(), token_in.clone())
        };

        for market in markets.iter() {
            // Only consider pools for this token pair
            let is_match = market.token_a == token_a && market.token_b == token_b;
            if !is_match {
                continue;
            }
            let Some(adapter_address) = adapters.get(market.adapter_id) else {
                continue;
            };
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

    fn swap(e: Env, amount: i128, token_in: Address, token_out: Address, best_hop: Address, sender: Address, deadline: u64) {
        // Find the adapter for the best_hop (pool address)
        let markets = get_markets(&e);
        let adapters = get_adapters(&e);

        // Look up the market by pool address
        let mut adapter_id: Option<i128> = None;
        for market in markets.iter() {
            if market.pool_address == best_hop {
                adapter_id = Some(market.adapter_id);
                break;
            }
        }

        let Some(adapter_id) = adapter_id else {
            panic_with_error!(&e, RouterError::PoolNotFound);
        };

        let Some(adapter_address) = adapters.get(adapter_id) else {
            panic_with_error!(&e, RouterError::InvalidID);
        };

        // Call adapter swap
        let adapter = AdapterClient::new(&e, &adapter_address);
        let path = Vec::from_array(&e, [token_in, token_out]);

        // AdapterClient auto-unwraps the result, so this returns i128 directly or panics
        adapter.swap_exact_in(&amount, &0i128, &path, &sender, &deadline);
    }

    fn provide_liquidity(
        e: Env,
        amount: i128,
        lp_plans: Vec<LpPlan>,
        sender: Address,
        deadline: u64,
    ) {
        // For each plan, call the appropriate adapter's add_liquidity
        let adapters = get_adapters(&e);
        for plan in lp_plans.iter() {
            let Some(adapter_address) = adapters.get(plan.adapter_id) else { continue; };
            let adapter = AdapterClient::new(&e, &adapter_address);
            // This assumes AdapterClient has add_liquidity implemented
            adapter.add_liquidity(
                &plan.token_a,
                &plan.token_b,
                &plan.amount_a,
                &plan.amount_b,
                &0i128,
                &0i128,
                &sender,
                &deadline,
            );
        }
    }

    fn redeem_liquidity(
        e: Env,
        lp_token: Address,
        lp_amount: i128,
        sender: Address,
        deadline: u64,
    ) {
        // Find the adapter for this lp_token
        let markets = get_markets(&e);
        let adapters = get_adapters(&e);
        let mut found = false;
        for market in markets.iter() {
            if market.lp_token == lp_token {
                let Some(adapter_address) = adapters.get(market.adapter_id) else { continue; };
                let adapter = AdapterClient::new(&e, &adapter_address);
                adapter.remove_liquidity(
                    &lp_token,
                    &lp_amount,
                    &0i128,
                    &0i128,
                    &sender,
                    &deadline,
                );
                found = true;
                break;
            }
        }
        if !found {
            panic!("No adapter found for lp_token");
        }
    }

    fn discover_soroswap_pools(e: Env, factory: Address, pairs_to_check: Vec<(Address, Address)>) {
        let soroswap_factory = soroswap_factory::Client::new(&e, &factory);
        let mut markets = get_markets(&e);

        for pair in pairs_to_check.iter() {
            let (token_a, token_b) = pair;
            let pair_address = soroswap_factory.get_pair(&token_a, &token_b);

            
            let soroswap_pair = soroswap_pair::Client::new(&e, &pair_address);
            let reserves = soroswap_pair.get_reserves();
            let market = MarketData {
                adapter_id: 3, // Soroswap PROTOCOL_ID
                pool_address: pair_address.clone(),
                lp_token: pair_address.clone(),
                token_a: token_a.clone(),
                token_b: token_b.clone(),
                reserve_a: reserves.0,
                reserve_b: reserves.1,
                pool_type: 0, // ConstantProduct
                ledger: e.ledger().sequence(),
            };
            markets.push_back(market);
        }
        set_markets(&e, &markets);
    }

    fn discover_aqua_pools(e: Env, factory: Address, pairs_to_check: Vec<(Address, Address)>) {
        // Similar logic to Soroswap, but using the Aqua factory and pair contracts
    }

    fn discover_phoenix_pools(e: Env, factory: Address, pairs_to_check: Vec<(Address, Address)>) {
        // Similar logic to Soroswap, but using the Phoenix factory and pair contracts
    }

    fn discover_comet_pools(e: Env, factory: Address, pairs_to_check: Vec<(Address, Address)>) {
        // Similar logic to Soroswap, but using the Comet factory and pair contracts
    }
}
