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
        // what is this init supposed to do i forgot. and it's not definied.
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
            // i don't know what this is supposed to be?  the error would get returned in the swap function not here right?  .map_err(|_| AdapterError::ExternalFailure)?;

        // Extract the final output amount (last element in the amounts vector)
        let amt_out = amounts
            .get(amounts.len() - 1)
            .ok_or(AdapterError::ExternalFailure)?;

        // this function isn't defined yet.
        swap( &e, SwapEvent {amt_in, amt_out, path, to,});
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
            // same issue i don't kno wwhat this is supposed to do it doesn't work.
            //.map_err(|_| AdapterError::ExternalFailure)?;

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
        to: Address,
        deadline: u64,
    ) -> Result<Address, AdapterError> {
        if !is_init(&e) {
            return Err(AdapterError::ExternalFailure);
        }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }

        // Call external router
        let router = SoroswapRouterClient::new(&e, &get_amm(&e)?);
        let (_amount_a, _amount_b, _liquidity) = router
            .add_liquidity(&a, &b, &amt_a, &amt_b, &0, &0, &to, &deadline);
            // and a third time the same thing is broken
            // .map_err(|_| AdapterError::ExternalFailure)?;

        // Get the LP token address by querying the factory
        // For now, we'll return the target address as a placeholder for the LP token
        // In a real implementation, you'd query the factory to get the actual pair address
        let lp_token = to.clone(); // This should be replaced with actual pair address lookup

        bump(&e);
        Ok(lp_token)
    }
    #[allow(unused_variables)]
    fn remove_liquidity(
        e: Env,
        lp: Address,
        lp_amt: i128,
        to: Address,
        deadline: u64,
    ) -> Result<(i128, i128), AdapterError> {
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
            .remove_liquidity(&token_a, &token_b, &lp_amt, &0, &0, &to, &deadline);
            // and again the 4th timem.
            // .map_err(|_| AdapterError::ExternalFailure)?;

        bump(&e);
        Ok((amt_a, amt_b))
    }
}
