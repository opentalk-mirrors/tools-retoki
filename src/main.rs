// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! `relbo` is a command-line tool to be used as a release helper bot in CI

#![warn(
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unnameable_types,
    unused_extern_crates,
    unused_qualifications,
    unused_results
)]

use clap::Parser;

use crate::{cli::Cli, command::Command, config::Config};

mod cli;
mod command;
mod config;
mod gitlab_service;
mod output;
mod tasks;
mod vcs_service;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = Config::load()?;

    cli.run(&config)
}
