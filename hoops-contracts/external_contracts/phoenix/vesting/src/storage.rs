#![allow(dead_code)]

use curve::Curve;
use phoenix::ttl::{
    INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL, PERSISTENT_RENEWAL_THRESHOLD,
    PERSISTENT_TARGET_TTL,
};
use soroban_sdk::{
    contracttype, log, panic_with_error, symbol_short, Address, ConversionError, Env, String,
    Symbol, TryFromVal, Val,
};

use crate::error::ContractError;

pub const ADMIN: Symbol = symbol_short!("ADMIN");
pub const VESTING_KEY: Symbol = symbol_short!("VESTING");
pub(crate) const PENDING_ADMIN: Symbol = symbol_short!("p_admin");

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Admin = 1,
    Config = 2,
    Minter = 3,
    Whitelist = 4,
    VestingTokenInfo = 5,
    MaxVestingComplexity = 6,
    IsInitialized = 7, //TODO: deprecated, remove in future upgrade
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub is_with_minter: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingTokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub address: Address,
}

// This structure is used as an argument during the vesting account creation
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub recipient: Address,
    pub curve: Curve,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingInfo {
    // the total amount of tokens left to be distributed
    // it's updated during each claim
    pub balance: u128,
    pub recipient: Address,
    pub schedule: Curve,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingCounterKey {
    pub recipient: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinterInfo {
    pub address: Address,
    pub mint_capacity: u128,
}

#[cfg(feature = "minter")]
impl MinterInfo {
    #[cfg(not(tarpaulin_include))]
    pub fn get_curve(&self) -> Curve {
        Curve::Constant(self.mint_capacity)
    }
}

pub fn save_admin_old(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
    env.storage().persistent().extend_ttl(
        &DataKey::Admin,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );
}

#[cfg(not(tarpaulin_include))]
pub fn _save_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
}

pub fn get_admin_old(env: &Env) -> Address {
    let admin_addr = env
        .storage()
        .persistent()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| {
            log!(&env, "Vesting: Get admin: Critical error - No admin found");
            panic_with_error!(env, ContractError::NoAdminFound);
        });
    env.storage().persistent().extend_ttl(
        &DataKey::Admin,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    admin_addr
}

#[cfg(not(tarpaulin_include))]
pub fn _get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    env.storage().instance().get(&ADMIN).unwrap_or_else(|| {
        log!(&env, "Vesting: Admin not set");
        panic_with_error!(&env, ContractError::AdminNotFound)
    })
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingInfoKey {
    pub recipient: Address,
    pub index: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingInfoResponse {
    pub balance: u128,
    pub recipient: Address,
    pub schedule: Curve,
    pub index: u64,
}

pub fn save_vesting(env: &Env, address: &Address, vesting_info: VestingInfo) {
    let counter_key = VestingCounterKey {
        recipient: address.clone(),
    };

    let next_index: u64 = env.storage().persistent().get(&counter_key).unwrap_or(0);

    let vesting_key = VestingInfoKey {
        recipient: address.clone(),
        index: next_index,
    };

    env.storage().persistent().set(&vesting_key, &vesting_info);
    env.storage().persistent().extend_ttl(
        &vesting_key,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    env.storage()
        .persistent()
        .set(&counter_key, &(next_index + 1));
}

pub fn update_vesting(env: &Env, address: &Address, index: u64, vesting_info: &VestingInfo) {
    let vesting_key = VestingInfoKey {
        recipient: address.clone(),
        index,
    };
    env.storage().persistent().set(&vesting_key, vesting_info);
    env.storage().persistent().extend_ttl(
        &vesting_key,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );
}

pub fn get_vesting(env: &Env, recipient: &Address, index: u64) -> VestingInfo {
    let vesting_key = VestingInfoKey {
        recipient: recipient.clone(),
        index,
    };
    let vesting_info = env.storage().persistent().get(&vesting_key).unwrap_or_else(|| {
        log!(&env, "Vesting: Get vesting schedule: Critical error - No vesting schedule found for the given address");
        panic_with_error!(env, ContractError::VestingNotFoundForAddress);
    });
    env.storage().persistent().extend_ttl(
        &vesting_key,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    vesting_info
}

//#[cfg(feature = "minter")]
pub fn save_minter(env: &Env, minter: &MinterInfo) {
    env.storage().instance().set(&DataKey::Minter, minter);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
}

//#[cfg(feature = "minter")]
pub fn get_minter(env: &Env) -> Option<MinterInfo> {
    use phoenix::ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL};

    let minter_info = env.storage().instance().get(&DataKey::Minter);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    minter_info
}

pub fn save_token_info(env: &Env, token_info: &VestingTokenInfo) {
    env.storage()
        .instance()
        .set(&DataKey::VestingTokenInfo, token_info);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
}

pub fn get_token_info(env: &Env) -> VestingTokenInfo {
    let vesting_token_info = env
        .storage()
        .instance()
        .get(&DataKey::VestingTokenInfo)
        .unwrap_or_else(|| {
            log!(
                &env,
                "Vesting: Get token info: Critical error - No token info found"
            );
            panic_with_error!(env, ContractError::NoTokenInfoFound);
        });
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    vesting_token_info
}

pub fn save_max_vesting_complexity(env: &Env, max_vesting_complexity: &u32) {
    env.storage()
        .instance()
        .set(&DataKey::MaxVestingComplexity, max_vesting_complexity);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
}

pub fn get_max_vesting_complexity(env: &Env) -> u32 {
    let vesting_complexity = env
        .storage()
        .instance()
        .get(&DataKey::MaxVestingComplexity)
        .unwrap();
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    vesting_complexity
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&DataKey::Config, &config);

    env.storage().persistent().extend_ttl(
        &DataKey::Config,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    )
}

pub fn get_config(env: &Env) -> Config {
    let config = env
        .storage()
        .persistent()
        .get(&DataKey::Config)
        .unwrap_or_else(|| {
            log!(&env, "Config not found");
            panic_with_error!(&env, ContractError::NoConfigFound)
        });

    env.storage().persistent().extend_ttl(
        &DataKey::Config,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    config
}
