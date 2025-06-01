#![no_std]
#![allow(dead_code)]

mod allowance;
mod balance;
mod contract;
pub mod errors;
mod interface;
mod metadata;
mod pool;
mod aqua_soroban_token_tests;
mod aqua_token_permissions_tests;
mod testutils;

pub use crate::contract::TokenClient;
