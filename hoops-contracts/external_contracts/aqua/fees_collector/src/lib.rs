#![no_std]

mod contract;
mod interface;
mod storage;
mod aqua_fees_collector_tests;
mod aqua_fees_collector_permissions_tests;
mod testutils;

pub use crate::contract::{FeesCollector, FeesCollectorClient};
