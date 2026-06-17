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
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

use crate::{cli::Cli, command::Command, config::Config};

mod cli;
mod command;
mod config;
mod gitlab_service;
mod output;
mod releases;
mod tasks;
mod templates;
mod vcs_service;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    init_tracing();

    let config = Config::load()?;

    cli.run(&config)
}

fn init_tracing() {
    let indicatif_layer = IndicatifLayer::new();
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("relbo=info,warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();
}
