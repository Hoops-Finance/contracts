#![no_std]

use soroban_sdk::{contractclient, contractspecfn, contracterror, Address, Env, Vec, BytesN};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AdapterError {
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
    // Add more as needed
}

pub struct Spec;

#[contractspecfn(name = "Spec", export = false)]
#[contractclient(name = "AdapterClient")]
pub trait AdapterTrait {
    /* -------- lifecycle ------------------------------------------------ */
    fn initialize(e: Env, amm_id: i128, amm_address: Address) -> Result<(), AdapterError>;
    fn upgrade (e: Env, new_wasm: BytesN<32>) -> Result<(), AdapterError>;
    fn version() -> u32;
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
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64,
    ) -> Result<(i128, i128, i128), AdapterError>;

    fn remove_liquidity(
        e: Env,
        lp_token: Address,
        lp_amount: i128,
        amt_a_min: i128,
        amt_b_min: i128,
        to: Address,
        deadline: u64,
    ) -> Result<(i128,i128), AdapterError>;
}
