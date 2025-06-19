#![no_std]

mod event;
mod protocol;
mod storage;

#[allow(unused_imports)]
use event::*;
use hoops_adapter_interface::{AdapterError, AdapterTrait};
use protocol::soroswap_pair::SoroswapPairClient;
use protocol::soroswap_router::SoroswapRouterClient;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};
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
            return Err(AdapterError::ExternalFailure);
        }
        if amm_id != PROTOCOL_ID {
            return Err(AdapterError::UnsupportedPair);
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
}
