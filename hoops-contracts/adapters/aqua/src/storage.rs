use soroban_sdk::{contracttype, Address, Env, Vec, BytesN,  Symbol, unwrap::UnwrapOptimized,symbol_short};
use hoops_adapter_interface::AdapterError;


#[derive(Clone)]
#[contracttype]
enum Key { Amm, Init, Pool(PoolKey), Lp(Address) }

const DAY_LEDGER: u32 = 17_280;
const BUMP: u32 = 60 * DAY_LEDGER;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreConfig {
    pub admin: Address,
    pub usdc: Address,
    pub next: u32,
    pub ttl_thresh: u32,
    pub ttl_bump: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolKey {
    pub tokens: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AquaPoolInfo {
    pub pool_address: Address,
    pub lp_token_address: Address,
}

pub const KEY_CORE_CONFIG: Symbol = symbol_short!("CONFIG");

pub fn set_core_config(e: &Env, config: &CoreConfig) {
    e.storage().instance().set(&KEY_CORE_CONFIG, config);
}
pub fn get_core_config(e: &Env) -> CoreConfig {
    e.storage().instance().get(&KEY_CORE_CONFIG).unwrap_optimized()
}
#[allow(dead_code)]
pub fn update_core_config<F: FnOnce(&mut CoreConfig)>(e: &Env, f: F) {
    let mut config = get_core_config(e);
    f(&mut config);
    set_core_config(e, &config);
}

pub fn set_amm(e:&Env, a:Address){ e.storage().instance().set(&Key::Amm,&a); }
pub fn get_amm(e:&Env)->Result<Address,AdapterError>{
    e.storage().instance().get(&Key::Amm).ok_or(AdapterError::ExternalFailure)
}
pub fn mark_init(e:&Env){ e.storage().instance().set(&Key::Init,&true); }

pub fn is_init(e:&Env)->bool{ e.storage().instance().has(&Key::Init) }

pub fn bump(e:&Env){
    e.storage().instance().extend_ttl(BUMP-DAY_LEDGER, BUMP);
}

fn sort_addresses(e: &Env, tokens: &Vec<Address>) -> Vec<Address> {
    let mut arr: [Address; 2] = [tokens.get_unchecked(0), tokens.get_unchecked(1)];
    if arr[0] > arr[1] {
        arr.swap(0, 1);
    }
    Vec::from_array(e, arr)
}

// Store a pool info for a given set of tokens (sorted for canonicalization)
pub fn set_pool_for_tokens(e: &Env, tokens: &Vec<Address>, info: &AquaPoolInfo) {
    let tokens_sorted = sort_addresses(e, tokens);
    let key = Key::Pool(PoolKey { tokens: tokens_sorted });
    e.storage().instance().set(&key, &info.clone());
    // Also store reverse mapping for LP token
    e.storage().instance().set(&Key::Lp(info.lp_token_address.clone()), &info.clone());
}

// Get a pool info for a given set of tokens (sorted for canonicalization)
pub fn get_pool_for_tokens(e: &Env, tokens: &Vec<Address>) -> Option<AquaPoolInfo> {
    let tokens_sorted = sort_addresses(e, tokens);
    let key = Key::Pool(PoolKey { tokens: tokens_sorted });
    e.storage().instance().get(&key)
}

// Get a pool info by LP token address
pub fn get_pool_by_lp_token(e: &Env, lp_token: &Address) -> Option<AquaPoolInfo> {
    e.storage().instance().get(&Key::Lp(lp_token.clone()))
}

