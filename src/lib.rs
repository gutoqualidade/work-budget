#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod bookkeeping;
pub use bookkeeping::{Cache, Jobs};
