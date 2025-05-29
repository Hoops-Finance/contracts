use soroban_sdk::{contracttype, Address, Env};
use hoops_adapter_interface::AdapterError;

#[derive(Clone)]
#[contracttype]
enum Key { Amm, Init }

const DAY_LEDGER: u32 = 17_280;
const BUMP: u32 = 60 * DAY_LEDGER;

pub fn set_amm(e:&Env, a:Address){ e.storage().instance().set(&Key::Amm,&a); }
pub fn get_amm(e:&Env)->Result<Address,AdapterError>{
    e.storage().instance().get(&Key::Amm).ok_or(AdapterError::ExternalFailure)
}
pub fn mark_init(e:&Env){ e.storage().instance().set(&Key::Init,&true); }
pub fn is_init(e:&Env)->bool{ e.storage().instance().has(&Key::Init) }

pub fn bump(e:&Env){
    e.storage().instance().extend_ttl(BUMP-DAY_LEDGER, BUMP);
}
