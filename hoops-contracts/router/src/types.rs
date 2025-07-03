use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolType {
    Soroswap = 0,
    Aqua = 1,
    Phoenix = 2,
    Comet = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreConfig {
    pub admin: Address,
    pub version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketData {
    pub adapter_id: i128,
    pub pool_address: Address,
    pub lp_token: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub reserve_a: i128,
    pub reserve_b: i128,
    pub pool_type: u32, // 0: ConstantProduct, 1: Stable, 2: Weighted
    pub ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LpPlan {
    pub token_a: Address,
    pub token_b: Address,
    pub amount_a: i128,
    pub amount_b: i128,
    pub proportion: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapQuote {
    pub adapter_id: i128,
    pub pool_address: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: i128,
    pub amount_out: i128,
    pub pool_type: u32,
    pub lp_token: Address,
}
