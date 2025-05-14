#![no_std]

use soroban_sdk::{contractclient, contractspecfn, Address, Env, Vec};
use hoops_common::CommonError;

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AdapterError {
    Common(CommonError as u32) = 100,
    UnsupportedPair            = 101,
    ExternalFailure            = 102,
}

pub struct Spec;

#[contractspecfn(name = "Spec", export = false)]
#[contractclient(name = "AdapterClient")]
pub trait AdapterTrait {
    /* -------- lifecycle ------------------------------------------------ */
    fn initialize(e: Env, amm_id: i128, amm_address: Address) -> Result<(), AdapterError>;
    fn upgrade   (e: Env, new_wasm: [u8;32])               -> Result<(), AdapterError>;

    /* -------- swaps ---------------------------------------------------- */
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

    /* -------- liquidity ------------------------------------------------ */
    fn add_liquidity(
        e: Env,
        token_a: Address,
        token_b: Address,
        amt_a: i128,
        amt_b: i128,
        to: Address,
        deadline: u64,
    ) -> Result<Address, AdapterError>;

    fn remove_liquidity(
        e: Env,
        lp_token: Address,
        lp_amount: i128,
        to: Address,
        deadline: u64,
    ) -> Result<(i128,i128), AdapterError>;
}
