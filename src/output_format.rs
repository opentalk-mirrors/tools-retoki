// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{io::Write, str::FromStr};

use anyhow::{Result, bail};
use clap::{Parser, ValueEnum};
use serde::Serialize;
use tabled::{
    Table, Tabled,
    grid::{
        config::ColoredConfig,
        dimension::CompleteDimension,
        records::vec_records::{Text, VecRecords},
    },
    settings::{Settings, Style, TableOption, Width, peaker::Priority},
};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Parser, ValueEnum)]
pub enum OutputFormat {
    /// Output the data in table format
    #[default]
    Table,

    /// Output the data in JSON format
    Json,

    /// Output the data in JSONL format (one JSON entity per line)
    Jsonl,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "jsonl" => Ok(Self::Jsonl),
            v => bail!("Unknown output format {v:?}"),
        }
    }
}

impl OutputFormat {
    pub fn output<D: Serialize + Tabled>(&self, data: &D) -> Result<()> {
        match self {
            OutputFormat::Table => {
                let table = Table::new([data]).with(table_style()).to_string();
                println!("{table}");
            }
            OutputFormat::Json => {
                serde_json::to_writer_pretty(std::io::stdout(), data)?;
            }
            OutputFormat::Jsonl => {
                let mut stdout = std::io::stdout().lock();
                serde_json::to_writer(&mut stdout, data)?;
                writeln!(stdout)?;
            }
        }
        Ok(())
    }

    pub fn output_multiple<D: Serialize + Tabled>(&self, data: &[D]) -> Result<()> {
        match self {
            OutputFormat::Table => {
                let table = Table::new(data).with(table_style()).to_string();
                println!("{table}");
            }
            OutputFormat::Json => {
                serde_json::to_writer_pretty(std::io::stdout(), data)?;
            }
            OutputFormat::Jsonl => {
                let mut stdout = std::io::stdout().lock();
                for entry in data {
                    serde_json::to_writer(&mut stdout, entry)?;
                    writeln!(stdout)?;
                }
            }
        }
        Ok(())
    }
}

/// Maximum table width in terminal columns. Cells exceeding this are wrapped so the table's borders
/// stay aligned instead of overflowing the terminal.
const MAX_TABLE_WIDTH: usize = 120;

/// Applies the rounded border and wraps overly wide cells so the table fits within
/// [`MAX_TABLE_WIDTH`], shrinking the widest column first.
fn table_style() -> impl TableOption<VecRecords<Text<String>>, ColoredConfig, CompleteDimension> {
    Settings::default().with(Style::rounded()).with(
        Width::wrap(MAX_TABLE_WIDTH)
            .keep_words(true)
            .priority(Priority::max(false)),
    )
}
