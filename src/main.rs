// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use command::Command;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, util::SubscriberInitExt as _};

mod command;
mod data;
mod helper;
mod output_format;
mod release_metadata;
mod template;

#[derive(Clone, Debug, PartialEq, Eq, Parser)]
#[command(author, version, about)]
struct Cli {
    /// The YAML file containing the structured release information.
    #[clap(long, default_value = "releases.yml", env = "RETOKI_RELEASE_FILE")]
    release_file: PathBuf,

    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    init_tracing();

    let Cli {
        release_file,
        command,
    } = Cli::parse();
    command.execute(release_file)
}

fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("retoki=info"));

    let indicatif_layer = IndicatifLayer::new();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time()
                .with_writer(indicatif_layer.get_stderr_writer()),
        )
        .with(indicatif_layer)
        .init();
}
