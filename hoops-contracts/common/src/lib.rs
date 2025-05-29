#![no_std]

use soroban_sdk::{contractclient, contractspecfn, Address, Env, Vec, BytesN, contracterror};

/// Adapter‑local error.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AdapterError {
    Common = 100,
    UnsupportedPair = 101,
    ExternalFailure = 102,
}


pub struct Spec;

/// Interface every AMM adapter MUST expose.
#[contractspecfn(name = "Spec", export = false)]
#[contractclient(name = "AdapterClient")]
pub trait Adapter {
    /* -------- lifecycle -------- */
    fn initialize(e: Env, amm_id: i128, amm_address: Address) -> Result<(), AdapterError>;
    fn upgrade   (e: Env, new_wasm: BytesN<32>) -> Result<(), AdapterError>;
    fn version() -> u32;
    /* -------- swaps -------- */
    fn swap_exact_in(
        e: Env,
        amount_in: i128,
        min_out: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<i128, AdapterError>;

    fn swap_exact_out(
        e: Env,
        amount_out: i128,
        max_in: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<i128, AdapterError>;

    /* -------- liquidity -------- */
    fn add_liquidity(
        e: Env,
        token_a: Address,
        token_b: Address,
        amt_a: i128,
        amt_b: i128,
        to: Address,
        deadline: u64,
    ) -> Result<Address, AdapterError>;               // LP token

    fn remove_liquidity(
        e: Env,
        lp_token: Address,
        lp_amount: i128,
        to: Address,
        deadline: u64,
    ) -> Result<(i128,i128), AdapterError>;           // (amt_a, amt_b)
}
