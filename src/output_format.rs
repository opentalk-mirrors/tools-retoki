// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{io::Write, str::FromStr};

use anyhow::{Result, bail};
use clap::{Parser, ValueEnum};
use serde::Serialize;
use tabled::{Table, Tabled, settings::Style};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Parser, ValueEnum)]
pub enum OutputFormat {
    /// Output the data in table format
    Table,

    /// Output the data in JSON format
    Json,

    /// Output the data in JSONL format (one JSON entity per line)
    Jsonl,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Table
    }
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
                let table = Table::new([data]).with(Style::markdown()).to_string();
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
                let table = Table::new(data).with(Style::markdown()).to_string();
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
