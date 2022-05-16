#![feature(drain_filter)]

extern crate ansi_term;
#[macro_use]
extern crate log;
extern crate loggerv;

pub mod common;

pub mod check;
pub mod convert;
pub mod parse;

pub mod pipeline;

#[cfg(test)]
mod test_util {
    // Manual include, otherwise, we have to make this part of the interface to make use of these
    // utility functions in tests. We don't want this to be part of the interface.
    include!("../tests/common.rs");
}
