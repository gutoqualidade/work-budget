// SPDX-License-Identifier: GPL-3.0-only
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod bookkeeping;
pub use bookkeeping::{Cache, Jobs};
