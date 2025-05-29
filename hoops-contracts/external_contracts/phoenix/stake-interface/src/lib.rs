#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Address, BytesN, String, Vec};

// Dummy types for interface only. Replace with real types if needed.
#[derive(Clone)]
pub struct Config;
#[derive(Clone)]
pub struct ConfigResponse;
#[derive(Clone)]
pub struct StakedResponse;
#[derive(Clone)]
pub struct AnnualizedRewardsResponse;
#[derive(Clone)]
pub struct WithdrawableRewardsResponse;
#[derive(Clone)]
pub struct WithdrawableReward;
#[derive(Clone)]
pub struct AnnualizedReward;
#[derive(Clone)]
pub struct ContractError;

#[contract]
pub struct Staking;

#[contractimpl]
impl Staking {
    pub fn bond(_env: Env, _sender: Address, _tokens: i128) {}
    pub fn unbond(_env: Env, _sender: Address, _stake_amount: i128, _stake_timestamp: u64) {}
    pub fn unbond_deprecated(_env: Env, _sender: Address, _stake_amount: i128, _stake_timestamp: u64) {}
    pub fn create_distribution_flow(_env: Env, _sender: Address, _asset: Address) {}
    pub fn distribute_rewards(_env: Env) {}
    pub fn withdraw_rewards(_env: Env, _sender: Address) {}
    pub fn withdraw_rewards_deprecated(_env: Env, _sender: Address) {}
    pub fn fund_distribution(
        _env: Env,
        _sender: Address,
        _start_time: u64,
        _distribution_duration: u64,
        _token_address: Address,
        _token_amount: i128,
    ) {}
    pub fn update_config(
        _env: Env,
        _lp_token: Option<Address>,
        _min_bond: Option<i128>,
        _min_reward: Option<i128>,
        _manager: Option<Address>,
        _owner: Option<Address>,
        _max_complexity: Option<u32>,
    ) -> Result<Config, ContractError> { Err(ContractError) }
    pub fn query_config(_env: Env) -> ConfigResponse { ConfigResponse }
    pub fn query_admin(_env: Env) -> Address { Address::generate(&_env) }
    pub fn query_staked(_env: Env, _address: Address) -> StakedResponse { StakedResponse }
    pub fn query_total_staked(_env: Env) -> i128 { 0 }
    pub fn query_annualized_rewards(_env: Env) -> AnnualizedRewardsResponse { AnnualizedRewardsResponse }
    pub fn query_withdrawable_rewards(_env: Env, _address: Address) -> WithdrawableRewardsResponse { WithdrawableRewardsResponse }
    pub fn query_withdrawable_rewards_dep(_env: Env, _address: Address) -> WithdrawableRewardsResponse { WithdrawableRewardsResponse }
    pub fn query_distributed_rewards(_env: Env, _asset: Address) -> u128 { 0 }
    pub fn query_undistributed_rewards(_env: Env, _asset: Address) -> u128 { 0 }
    pub fn propose_admin(
        _env: Env,
        _new_admin: Address,
        _time_limit: Option<u64>,
    ) -> Result<Address, ContractError> { Err(ContractError) }
    pub fn revoke_admin_change(_env: Env) -> Result<(), ContractError> { Err(ContractError) }
    pub fn accept_admin(_env: Env) -> Result<Address, ContractError> { Err(ContractError) }
    pub fn migrate_admin_key(_env: Env) -> Result<(), ContractError> { Err(ContractError) }
}
