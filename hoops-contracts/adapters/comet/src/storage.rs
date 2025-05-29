use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, unwrap::UnwrapOptimized};
use hoops_adapter_interface::AdapterError;

#[derive(Clone)]
#[contracttype]
enum Key { Amm, Init }

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
