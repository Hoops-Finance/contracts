#![no_std]

use soroban_sdk::{contractclient, contractspecfn, contracterror, contracttype, Address, Env, Vec, BytesN};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AdapterError {
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
    InsufficientLiquidity = 212,
    PairNotFound = 213,
    InvalidPool = 214,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolInfo {
    pub pool_address: Address,
    pub lp_token: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub pool_type: u32,
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


    /* ---------- quotes ---------- */
    fn quote_in(e: Env, pool_address: Address, amount_in: i128, token_in: Address, token_out: Address) -> Result<i128, AdapterError>;
    fn quote_out(e: Env, pool_address: Address, amount_out: i128, token_in: Address, token_out: Address) -> Result<i128, AdapterError>;

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

    /* -------- direct pool withdrawal (called by Router) --------------- */
    /// Withdraw liquidity directly from a pool. LP tokens are pre-transferred
    /// to the adapter by the Router. The adapter handles protocol-specific
    /// withdrawal and sends reserve tokens to `to` (the Account).
    fn remove_liq_from_pool(
        e: Env,
        pool: Address,
        lp_amount: i128,
        to: Address,
    ) -> (i128, i128);

    /* -------- pool validation (called by Router for lazy registration) -- */
    /// Validate that a pool address is a legitimate pool on this protocol
    /// for the given token pair. Returns pool metadata on success.
    fn validate_pool(
        e: Env,
        pool_address: Address,
        token_a: Address,
        token_b: Address,
    ) -> Result<PoolInfo, AdapterError>;
}
