#![no_std]

mod event;
mod protocol;
mod storage;

#[allow(unused_imports)]
use event::*;
use hoops_adapter_interface::{AdapterError, AdapterTrait};
use protocol::soroswap_pair::SoroswapPairClient;
use protocol::soroswap_router::SoroswapRouterClient;
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, Vec};
use storage::*;

const PROTOCOL_ID: i128 = 3;

#[contract]
pub struct SoroswapAdapter;

#[contractimpl]
impl AdapterTrait for SoroswapAdapter {
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), AdapterError> {
        // setup the config to get the admin address. set it during initialization.
        let config = get_core_config(&e);
        config.admin.require_auth();
        Ok(e.deployer().update_current_contract_wasm(new_wasm_hash))
    }

    fn version() -> u32 {
        1 // Version 1 of the Soroswap adapter
    }

    /* ---------- lifecycle ---------- */
    fn initialize(e: Env, amm_id: i128, amm_addr: Address) -> Result<(), AdapterError> {
        if is_init(&e) {
            return Err(AdapterError::AlreadyInitialized);
        }
        if amm_id != PROTOCOL_ID {
            return Err(AdapterError::InvalidID);
        }

        set_amm(&e, amm_addr.clone());
        mark_init(&e);
        bump(&e);
        
        init(&e, amm_addr);
        Ok(())
    }

    /* ---------- swaps ---------- */
    fn swap_exact_in(
        e: Env,
        amt_in: i128,
        min_out: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<i128, AdapterError> {
        to.require_auth();
        if !is_init(&e) {
            return Err(AdapterError::ExternalFailure);
        }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        // Call external router
        let router = SoroswapRouterClient::new(&e, &get_amm(&e)?);
        let amounts = router
            .swap_exact_tokens_for_tokens(&amt_in, &min_out, &path, &to, &deadline);

        // Extract the final output amount (last element in the amounts vector)
        let amt_out = amounts
            .get(amounts.len() - 1)
            .ok_or(AdapterError::ExternalFailure)?;

        swap(
            &e,
            SwapEvent {
                amt_in,
                amt_out,
                path,
                to,
            },
        );
        bump(&e);
        Ok(amt_out)
    }

    fn swap_exact_out(
        e: Env,
        out: i128,
        max_in: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<i128, AdapterError> {
        to.require_auth();
        if !is_init(&e) {
            return Err(AdapterError::ExternalFailure);
        }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }

        // Call external router
        let router = SoroswapRouterClient::new(&e, &get_amm(&e)?);
        let amounts = router
            .swap_tokens_for_exact_tokens(&out, &max_in, &path, &to, &deadline);

        // Extract the input amount (first element in the amounts vector)
        let amt_in = amounts.get(0).ok_or(AdapterError::ExternalFailure)?;

        swap(
            &e,
            SwapEvent {
                amt_in,
                amt_out: out,
                path,
                to,
            },
        );
        bump(&e);
        Ok(amt_in)
    }

    /* ---------- liquidity ---------- */
    #[allow(unused_variables)]
    fn add_liquidity(
        e: Env,
        a: Address,
        b: Address,
        amt_a: i128,
        amt_b: i128,
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64,
    ) -> Result<(i128, i128, i128), AdapterError> {
        to.require_auth();
        if !is_init(&e) {
            return Err(AdapterError::ExternalFailure);
        }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        // Call external router to add liquidity
        let router = SoroswapRouterClient::new(&e, &get_amm(&e)?);
        let (amount_a, amount_b, liquidity) = router
            .add_liquidity(&a, &b, &amt_a, &amt_b, &amt_a_min, &amt_b_min, &to, &deadline);
        bump(&e);
        Ok((amount_a, amount_b, liquidity))
    }

    #[allow(unused_variables)]
    fn remove_liquidity(
        e: Env,
        lp: Address,
        lp_amt: i128,
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64,
    ) -> Result<(i128, i128), AdapterError> {
        to.require_auth();
        if !is_init(&e) {
            return Err(AdapterError::ExternalFailure);
        }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        // Get the underlying tokens from the LP token (pair contract)
        let pair_client = SoroswapPairClient::new(&e, &lp);
        let token_a = pair_client.token_0();
        let token_b = pair_client.token_1();
        // Call external router
        let router = SoroswapRouterClient::new(&e, &get_amm(&e)?);
        let (amt_a, amt_b) = router
            .remove_liquidity(&token_a, &token_b, &lp_amt, &amt_a_min, &amt_b_min, &to, &deadline);
        bump(&e);
        Ok((amt_a, amt_b))
    }

    /* ---------- quotes ---------- */
    fn quote_in(e: Env, pool_address: Address, amount_in: i128, token_in: Address, token_out: Address) -> Result<i128, AdapterError> {
        if !is_init(&e) {
            return Err(AdapterError::NotInitialized);
        }
        if amount_in <= 0 {
            return Err(AdapterError::InvalidAmount);
        }
        let pair = protocol::soroswap_pair::SoroswapPairClient::new(&e, &pool_address);
        let (reserve_0, reserve_1) = pair.get_reserves();
        // Canonical token order
        let (reserve_in, reserve_out) = if token_in < token_out {
            (reserve_0 as u128, reserve_1 as u128)
        } else {
            (reserve_1 as u128, reserve_0 as u128)
        };
        let amount_in_u128 = amount_in as u128;
        if reserve_in == 0 || reserve_out == 0 {
            return Ok(0);
        }
        // Apply 0.3% fee: amount_in_with_fee = amount_in * 997 / 1000
        let amount_in_with_fee = amount_in_u128 * 997;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * 1000 + amount_in_with_fee;
        let amount_out = numerator / denominator;
        Ok(amount_out as i128)
    }

    fn quote_out(e: Env, pool_address: Address, amount_out: i128, token_in: Address, token_out: Address) -> Result<i128, AdapterError> {
        if !is_init(&e) {
            return Err(AdapterError::NotInitialized);
        }
        if amount_out <= 0 {
            return Err(AdapterError::InvalidAmount);
        }
        let pair = protocol::soroswap_pair::SoroswapPairClient::new(&e, &pool_address);
        let (reserve_0, reserve_1) = pair.get_reserves();
        let (reserve_in, reserve_out) = if token_in < token_out {
            (reserve_0 as u128, reserve_1 as u128)
        } else {
            (reserve_1 as u128, reserve_0 as u128)
        };
        let amount_out_u128 = amount_out as u128;
        if reserve_in == 0 || reserve_out == 0 || reserve_out <= amount_out_u128 {
            return Err(AdapterError::InsufficientLiquidity);
        }
        // Reverse formula for required input, including fee
        let numerator = reserve_in * amount_out_u128 * 1000;
        let denominator = (reserve_out - amount_out_u128) * 997;
        let amount_in = numerator / denominator + 1;
        Ok(amount_in as i128)
    }
}

/// Direct pair swap - called by the Router when tokens are pre-transferred.
/// Keeps auth chains shallow (DeFindex pattern): each contract operates
/// within its own authorization context.
#[contractimpl]
impl SoroswapAdapter {
    pub fn swap_in_pool(
        e: Env,
        amt_in: i128,
        min_out: i128,
        token_in: Address,
        token_out: Address,
        pool: Address,
        to: Address,
    ) -> i128 {
        // Tokens are already in this adapter (transferred by Router).
        // Transfer from adapter to the pair - adapter is direct caller → auth works.
        token::Client::new(&e, &token_in)
            .transfer(&e.current_contract_address(), &pool, &amt_in);

        // Query pair reserves and determine token ordering
        let pair = SoroswapPairClient::new(&e, &pool);
        let (reserve_0, reserve_1) = pair.get_reserves();
        let token_0 = pair.token_0();

        let (reserve_in, reserve_out) = if token_in == token_0 {
            (reserve_0, reserve_1)
        } else {
            (reserve_1, reserve_0)
        };

        // Calculate output with 0.3% fee (constant product x*y=k)
        let amt_u = amt_in as u128;
        let fee_adj = amt_u * 997;
        let amount_out = (fee_adj * reserve_out as u128
            / (reserve_in as u128 * 1000 + fee_adj)) as i128;

        assert!(amount_out >= min_out, "slippage");

        // Call pair.swap() - pair requires NO auth from `to`,
        // it just validates the K invariant and sends output tokens.
        let (a0, a1) = if token_in == token_0 {
            (0i128, amount_out)
        } else {
            (amount_out, 0i128)
        };
        pair.swap(&a0, &a1, &to);

        // Emit swap event
        swap(
            &e,
            SwapEvent {
                amt_in,
                amt_out: amount_out,
                path: Vec::from_array(&e, [token_in, token_out]),
                to,
            },
        );
        bump(&e);

        amount_out
    }

    /// Direct pair liquidity deposit - called by the Router when tokens are pre-transferred.
    /// Same shallow-auth pattern as swap_in_pool.
    pub fn add_liq_in_pool(
        e: Env,
        token_a: Address,
        token_b: Address,
        amount_a: i128,
        amount_b: i128,
        pool: Address,
        to: Address,
    ) -> i128 {
        // Tokens are already in this adapter (transferred by Router).
        // Transfer both tokens from adapter to the pair.
        token::Client::new(&e, &token_a)
            .transfer(&e.current_contract_address(), &pool, &amount_a);
        token::Client::new(&e, &token_b)
            .transfer(&e.current_contract_address(), &pool, &amount_b);

        // Call pair.deposit(to) - mints LP tokens based on balance delta.
        // pair.deposit() does NOT require auth from `to`.
        let pair = SoroswapPairClient::new(&e, &pool);
        let lp_minted = pair.deposit(&to);

        bump(&e);
        lp_minted
    }
}
