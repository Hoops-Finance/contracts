#![no_std]

pub mod c_consts;
pub mod c_math;
pub mod c_num;
pub mod c_pool;

#[cfg(test)]
mod comet_pool_tests;

#[cfg(test)]
extern crate std;
