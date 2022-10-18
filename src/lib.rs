//! This crate contains common types used for memory mapping. 

// #![no_std]
#![feature(step_trait)]

// extern crate bit_field;
#[macro_use] extern crate derive_more;
// extern crate prusti_contracts;

pub mod addr;
pub mod unit;
pub mod range;
pub mod range_inclusive;
mod test;

fn main() {}