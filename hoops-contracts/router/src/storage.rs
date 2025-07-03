use soroban_sdk::{contracttype, Env, Address, Vec, Map};
use crate::types::{CoreConfig, MarketData};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    CoreConfig,
    Adapters, // Map<i128, Address>
    Markets, // Vec<MarketData>
}

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

pub fn get_markets(e: &Env) -> Vec<MarketData> {
    e.storage().instance().get(&DataKey::Markets).unwrap_or(Vec::new(e))
}

pub fn set_markets(e: &Env, markets: &Vec<MarketData>) {
    e.storage().instance().set(&DataKey::Markets, markets);
}
