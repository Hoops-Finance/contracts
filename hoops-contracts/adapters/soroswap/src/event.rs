#![allow(dead_code)]

use soroban_sdk::{contracttype, symbol_short, Address, Env, Vec};

#[contracttype]
#[derive(Clone)]
pub struct InitEvent { pub amm: Address }

#[contracttype]
#[derive(Clone)]
pub struct SwapEvent { pub amt_in: i128, pub amt_out: i128, pub path: Vec<Address>, pub to: Address }

#[contracttype]
#[derive(Clone)]
pub struct AddLpEvent { pub token_a: Address, pub token_b: Address, pub lp: Address, pub to: Address }

#[contracttype]
#[derive(Clone)]
pub struct RemLpEvent { pub lp: Address, pub to: Address }

pub(crate) fn init(e: &Env, amm: Address) { e.events().publish(("soroswap",symbol_short!("init")), InitEvent{amm}); }
pub(crate) fn swap(e:&Env, ev:SwapEvent){ e.events().publish(("soroswap",symbol_short!("swap")), ev); }

pub(crate) fn add_lp(e:&Env, ev:AddLpEvent){ e.events().publish(("soroswap",symbol_short!("addlp")), ev); }
pub(crate) fn rem_lp(e:&Env, ev:RemLpEvent){ e.events().publish(("soroswap",symbol_short!("remlp")), ev); }
