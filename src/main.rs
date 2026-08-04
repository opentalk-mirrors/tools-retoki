// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use clap::{CommandFactory as _, Parser};
use clap_complete::env::CompleteEnv;
use command::Cli;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, util::SubscriberInitExt as _};

mod bot_config;
mod bot_templates;
mod command;
mod completion;
mod data;
mod gitlab_service;
mod helper;
mod output;
mod output_format;
mod release_metadata;
mod release_workflow;
mod tasks;
mod template;
mod vcs_service;

fn main() -> Result<()> {
    // Must run before anything writes to stdout: handles completion requests and exits.
    CompleteEnv::with_factory(Cli::command).complete();

    init_tracing();
    let cli = Cli::parse();
    cli.command.execute(cli.release_file)
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
