use soroban_sdk::{contracttype, Env, Address, Vec, Map};
use crate::types::{CoreConfig, MarketData};

const DAY_LEDGER: u32 = 17_280;
const TTL_THRESHOLD: u32 = 30 * DAY_LEDGER;
const TTL_BUMP: u32 = 60 * DAY_LEDGER;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    CoreConfig,
    Adapters,
    Pool(Address),
    TokenPair(TokenPairKey),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenPairKey {
    pub token_a: Address,
    pub token_b: Address,
}

// --- Instance storage (CoreConfig, Adapters) ---

pub fn get_core_config(e: &Env) -> CoreConfig {
    e.storage().instance().get(&DataKey::CoreConfig).unwrap()
}

pub fn set_core_config(e: &Env, config: &CoreConfig) {
    e.storage().instance().set(&DataKey::CoreConfig, config);
}

pub fn get_adapters(e: &Env) -> Map<i128, Address> {
    e.storage().instance().get(&DataKey::Adapters).unwrap_or(Map::new(e))
}

pub fn set_adapters(e: &Env, adapters: &Map<i128, Address>) {
    e.storage().instance().set(&DataKey::Adapters, adapters);
}

// --- Persistent storage (per-pool) ---

pub fn get_pool(e: &Env, pool_address: &Address) -> Option<MarketData> {
    let key = DataKey::Pool(pool_address.clone());
    let val: Option<MarketData> = e.storage().persistent().get(&key);
    if val.is_some() {
        e.storage().persistent().extend_ttl(&key, TTL_THRESHOLD, TTL_BUMP);
    }
    val
}

pub fn set_pool(e: &Env, pool_address: &Address, market: &MarketData) {
    let key = DataKey::Pool(pool_address.clone());
    e.storage().persistent().set(&key, market);
    e.storage().persistent().extend_ttl(&key, TTL_THRESHOLD, TTL_BUMP);
}

pub fn has_pool(e: &Env, pool_address: &Address) -> bool {
    e.storage().persistent().has(&DataKey::Pool(pool_address.clone()))
}

// --- Persistent storage (token-pair index) ---

fn canonical_pair_key(token_a: &Address, token_b: &Address) -> TokenPairKey {
    if token_a < token_b {
        TokenPairKey { token_a: token_a.clone(), token_b: token_b.clone() }
    } else {
        TokenPairKey { token_a: token_b.clone(), token_b: token_a.clone() }
    }
}

pub fn get_pools_for_pair(e: &Env, token_a: &Address, token_b: &Address) -> Vec<Address> {
    let pair_key = canonical_pair_key(token_a, token_b);
    let key = DataKey::TokenPair(pair_key);
    let val: Vec<Address> = e.storage().persistent().get(&key).unwrap_or(Vec::new(e));
    if !val.is_empty() {
        e.storage().persistent().extend_ttl(&key, TTL_THRESHOLD, TTL_BUMP);
    }
    val
}

pub fn add_pool_to_pair_index(e: &Env, token_a: &Address, token_b: &Address, pool_address: &Address) {
    let pair_key = canonical_pair_key(token_a, token_b);
    let key = DataKey::TokenPair(pair_key);
    let mut pools: Vec<Address> = e.storage().persistent().get(&key).unwrap_or(Vec::new(e));
    // Check for duplicates
    for p in pools.iter() {
        if p == *pool_address {
            return;
        }
    }
    pools.push_back(pool_address.clone());
    e.storage().persistent().set(&key, &pools);
    e.storage().persistent().extend_ttl(&key, TTL_THRESHOLD, TTL_BUMP);
}
