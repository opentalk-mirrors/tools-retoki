//! `relbo` is a command-line tool to be used as a release helper bot in CI

#![deny(
    bad_style,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_bounds,
    private_interfaces,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unnameable_types,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    unused_results,
    while_true
)]

use clap::Parser;
use snafu::Whatever;

use crate::{cli::Cli, config::Config};

mod cli;
mod config;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let cli = Cli::parse();

    println!("command-line arguments: {cli:?}");

    let config = Config::load()?;
    println!("config: {config:?}");

    Ok(())
}
