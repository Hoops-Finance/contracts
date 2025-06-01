#![no_std]

mod constants;
mod contract;
mod errors;
mod events;
mod interface;
mod storage;
mod aqua_lp_provider_swap_fee_tests;
mod testutils;

pub use crate::contract::{ProviderSwapFeeCollector, ProviderSwapFeeCollectorClient};
