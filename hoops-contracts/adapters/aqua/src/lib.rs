#![no_std]

mod storage;
mod event;
mod protocol;

use storage::*;
use event::*;
use hoops_adapter_interface::{AdapterTrait, AdapterError};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};
use protocol::AquaRouterClient;

const PROTOCOL_ID: i128 = 0;

#[contract]
pub struct AquaAdapter;

#[contractimpl]
impl AdapterTrait for AquaAdapter {

    fn version() -> u32 {
        1 // Version 1 of the Aqua adapter
    }
    /* ---------- lifecycle ---------- */
    fn initialize(e: Env, amm_id: i128, amm_addr: Address) -> Result<(), AdapterError> {
        if is_init(&e) { return Err(AdapterError::ExternalFailure); }
        if amm_id != PROTOCOL_ID { return Err(AdapterError::UnsupportedPair); }

        set_amm(&e, amm_addr.clone());
        mark_init(&e);
        bump(&e);
        init(&e, amm_addr);
        Ok(())
    }

    fn upgrade(_e: Env, _hash: BytesN<32>) -> Result<(), AdapterError> {
        // Stub: always fail for now
        Err(AdapterError::ExternalFailure)
    }

    /* ---------- swaps ---------- */
    fn swap_exact_in(
        e: Env,
        amt_in: i128,
        min_out: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64
    ) -> Result<i128, AdapterError> {

        if !is_init(&e){ return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp()>deadline{
            return Err(AdapterError::ExternalFailure);
        }

        // Call external router with Aqua's swap_chained method
        let router = AquaRouterClient::new(&e, &get_amm(&e)?);
        
        // Convert path to Aqua's swaps_chain format
        let mut swaps_chain: Vec<(Vec<Address>, BytesN<32>, Address)> = Vec::new(&e);
        for i in 0..(path.len() - 1) {
            let token_in = path.get(i).unwrap();
            let token_out = path.get(i + 1).unwrap();
            let mut tokens = Vec::new(&e);
            tokens.push_back(token_in);
            tokens.push_back(token_out.clone());
            let pool_index = BytesN::from_array(&e, &[0; 32]); // Default pool index
            swaps_chain.push_back((tokens, pool_index, token_out));
        }
        
        let amt_out = router.swap_chained(
            &to,
            &swaps_chain,
            &path.get(0).unwrap(),
            &(amt_in as u128),
            &(min_out as u128)
        );
        let amt_out_i128 = amt_out as i128;

        swap(&e, SwapEvent{ amt_in, amt_out: amt_out_i128, path, to});
        bump(&e);
        Ok(amt_out_i128)
    }

    fn swap_exact_out(
        e: Env, out: i128, max_in: i128, path: Vec<Address>,
        to: Address, deadline: u64
    ) -> Result<i128, AdapterError> {
        if !is_init(&e){ return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp()>deadline{
            return Err(AdapterError::ExternalFailure);
        }
        
        // Call external router with Aqua's swap_chained_strict_receive method
        let router = protocol::AquaRouterClient::new(&e, &get_amm(&e)?);
        
        // Convert path to Aqua's swaps_chain format
        let mut swaps_chain: Vec<(Vec<Address>, BytesN<32>, Address)> = Vec::new(&e);
        for i in 0..(path.len() - 1) {
            let token_in = path.get(i).unwrap();
            let token_out = path.get(i + 1).unwrap();
            let mut tokens = Vec::new(&e);
            tokens.push_back(token_in);
            tokens.push_back(token_out.clone());
            let pool_index = BytesN::from_array(&e, &[0; 32]); // Default pool index
            swaps_chain.push_back((tokens, pool_index, token_out));
        }
        
        let amt_in = router.swap_chained_strict_receive(
            &to,
            &swaps_chain,
            &path.get(0).unwrap(),
            &(out as u128),
            &(max_in as u128)
        );
        let amt_in_i128 = amt_in as i128;

        swap(&e, SwapEvent{ amt_in: amt_in_i128, amt_out: out, path, to });
        bump(&e);
        Ok(amt_in_i128)
    }

    /* ---------- liquidity ---------- */
    fn add_liquidity(
        e: Env,
        a: Address,
        b: Address,
        amt_a: i128,
        amt_b: i128,
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64
    ) -> Result<(i128, i128, i128), AdapterError> {
        if !is_init(&e) { return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        let router = protocol::AquaRouterClient::new(&e, &get_amm(&e)?);
        let mut tokens = Vec::new(&e);
        tokens.push_back(a.clone());
        tokens.push_back(b.clone());
        let mut desired_amounts = Vec::new(&e);
        desired_amounts.push_back(amt_a as u128);
        desired_amounts.push_back(amt_b as u128);
        let mut min_amounts = Vec::new(&e);
        min_amounts.push_back(amt_a_min as u128);
        min_amounts.push_back(amt_b_min as u128);
        let pool_index = BytesN::from_array(&e, &[0; 32]);
        let min_shares = 1u128; // Minimum shares to accept
        let (amounts, shares) = router.deposit(
            &to,
            &tokens,
            &pool_index,
            &desired_amounts,
            &min_shares
        );
        let amount_a = amounts.get(0).unwrap() as i128;
        let amount_b = amounts.get(1).unwrap() as i128;
        let shares_i128 = shares as i128;
        bump(&e);
        Ok((amount_a, amount_b, shares_i128))
    }
#[allow(unused_variables)]
    fn remove_liquidity(
        e: Env,
        lp: Address,
        lp_amt: i128,
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64
    ) -> Result<(i128, i128), AdapterError> {
        if !is_init(&e) { return Err(AdapterError::ExternalFailure); }
        if e.ledger().timestamp() > deadline {
            return Err(AdapterError::ExternalFailure);
        }
        let router = protocol::AquaRouterClient::new(&e, &get_amm(&e)?);
        let tokens = Vec::new(&e);
        // Would need to get actual tokens from pool contract
        let pool_index = BytesN::from_array(&e, &[0; 32]);
        let mut min_amounts = Vec::new(&e);
        min_amounts.push_back(amt_a_min as u128);
        min_amounts.push_back(amt_b_min as u128);
        let amounts = router.withdraw(
            &to,
            &tokens,
            &pool_index,
            &(lp_amt as u128),
            &min_amounts
        );
        let amt_a = if amounts.len() > 0 { amounts.get(0).unwrap() as i128 } else { 0 };
        let amt_b = if amounts.len() > 1 { amounts.get(1).unwrap() as i128 } else { 0 };
        bump(&e);
        Ok((amt_a, amt_b))
    }
}
