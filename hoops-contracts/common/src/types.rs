use soroban_sdk::{contracttype, Address};
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhoStakeInitInfo {
    pub min_bond: i128,
    pub min_reward: i128,
    pub manager: Address,
    pub max_complexity: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhoTokenInitInfo {
    pub token_a: Address,
    pub token_b: Address,
}

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PhoenixPoolType {
    Constant = 0,
    Stable = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhoLiquidityPoolInitInfo {
    pub admin: Address,
    pub swap_fee_bps: i64,
    pub fee_recipient: Address,
    pub max_allowed_slippage_bps: i64,
    pub default_slippage_bps: i64,
    pub max_allowed_spread_bps: i64,
    pub max_referral_bps: i64,
    pub token_init_info: PhoTokenInitInfo,
    pub stake_init_info: PhoStakeInitInfo,
}
