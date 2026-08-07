// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use clap_complete::{ArgValueCandidates, Shell};

use self::{
    ci::CiArgs, compare::CompareArgs, edit::EditArgs, generate::GenerateArgs, release::ReleaseArgs,
    series::SeriesArgs,
};

pub mod ci;
pub mod compare;
pub mod edit;
pub mod generate;
pub mod release;
pub mod series;
pub mod utils;

/// Long flag name (without leading `--`) of the `release_file` argument.
pub const RELEASE_FILE_FLAG: &str = "release-file";
/// Default file name used for the `release_file` argument.
pub const DEFAULT_RELEASE_FILE: &str = "releases.yml";

#[derive(Clone, Debug, PartialEq, Eq, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// The YAML file containing the structured release information.
    #[clap(long = RELEASE_FILE_FLAG, default_value = DEFAULT_RELEASE_FILE, env = "RETOKI_RELEASE_FILE")]
    pub release_file: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Generate the release information from a `releases.yml` file
    Generate(GenerateArgs),

    /// Perform actions related to a release
    Release(ReleaseArgs),

    /// Perform actions related to a release series
    Series(SeriesArgs),

    /// Run tasks intended to be executed in a CI pipeline
    ///
    /// This currently updates the dependency graphs in all open release tickets.
    Ci(CiArgs),

    /// Edit a `releases.yml` file
    Edit(EditArgs),

    /// Compare the `releases.yml` file with another `releases.yml` file
    ///
    /// This will print to stdout one line for each release that is found in either of the
    /// `releases.yml` files
    ///
    /// A release was added from the other to the current `releases.yml` file:
    /// `+ <version>`
    ///
    /// A release was removed from the other to the current `releases.yml` file:
    /// `- <version>`
    ///
    /// A release is present in both `releases.yml` files and was unchanged:
    /// `= <version>`
    ///
    /// A release is present in both `releases.yml` files and was changed:
    /// `~ <version>`
    #[clap(verbatim_doc_comment)]
    Compare(CompareArgs),

    /// Set up shell completions for the current user
    ///
    /// Completions are computed dynamically by the `retoki` binary itself and
    /// installed as a completion file in the shell's autoloaded completion
    /// directory. Only Bash, Zsh, and Fish are supported. Restart your shell
    /// afterwards. Auto-detects the shell from $SHELL.
    InstallCompletions {
        /// Override the detected shell
        #[clap(long)]
        shell: Option<Shell>,
    },
}

impl Command {
    pub fn execute<R: AsRef<Path>>(self, release_file: R) -> Result<()> {
        match self {
            Command::Generate(args) => args.execute(release_file),
            Command::Release(args) => args.execute(release_file),
            Command::Series(args) => args.execute(release_file),
            Command::Edit(args) => args.execute(release_file),
            Command::Compare(args) => args.execute(release_file),
            Command::Ci(args) => args.run(),
            Command::InstallCompletions { shell } => install_completions(shell),
        }
    }
}

fn install_completions(shell_override: Option<Shell>) -> Result<()> {
    let shell = shell_override
        .or_else(Shell::from_env)
        .context("could not detect the shell; use --shell to specify one")?;

    let target = completion_target(shell)?;
    install_completion_file(shell, target)
}

/// Where and how a shell's generated completion script is installed as a file.
struct CompletionTarget {
    /// Full path of the completion file to write.
    path: PathBuf,
    /// Extra line appended to the generated script. zsh needs a dispatch call
    /// because `compinit` autoloads the file under its name, not the inner
    /// function.
    dispatch: Option<String>,
    /// A `(startup file, line)` to append so the completion directory is found.
    /// `None` when the shell scans the directory on its own.
    activation: Option<(PathBuf, String)>,
}

/// The completion file to install for `shell`.
///
/// Only bash, zsh, and fish are supported; any other shell is an error.
fn completion_target(shell: Shell) -> Result<CompletionTarget> {
    let bin = env!("CARGO_PKG_NAME");

    let target = match shell {
        Shell::Zsh => {
            let dir = xdg_data_home()?.join("zsh/site-functions");
            CompletionTarget {
                path: dir.join(format!("_{bin}")),
                dispatch: Some(format!("_clap_dynamic_completer_{bin} \"$@\"")),
                activation: Some((
                    home_dir()?.join(".zshrc"),
                    format!(
                        "fpath=(\"{}\" $fpath); autoload -Uz compinit && compinit",
                        dir.display()
                    ),
                )),
            }
        }
        Shell::Bash => CompletionTarget {
            path: xdg_data_home()?
                .join("bash-completion/completions")
                .join(bin),
            dispatch: None,
            activation: None,
        },
        Shell::Fish => CompletionTarget {
            path: xdg_config_home()?
                .join("fish/completions")
                .join(format!("{bin}.fish")),
            dispatch: None,
            activation: None,
        },
        other => anyhow::bail!(
            "shell completions are only supported for bash, zsh, and fish (got {other})"
        ),
    };

    Ok(target)
}

/// Install a generated completion script as a file in the shell's autoloaded
/// completion directory.
fn install_completion_file(shell: Shell, target: CompletionTarget) -> Result<()> {
    if let Some(parent) = target.path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let mut script = generated_script(shell)?;
    if let Some(dispatch) = &target.dispatch {
        if !script.ends_with('\n') {
            script.push('\n');
        }
        script.push_str(dispatch);
        script.push('\n');
    }

    fs::write(&target.path, script)
        .with_context(|| format!("failed to write {}", target.path.display()))?;

    let mut activated = None;
    if let Some((rc, line)) = &target.activation
        && append_line_if_missing(rc, line)?
    {
        activated = Some(rc.clone());
    }

    eprintln!(
        "Installed {shell} completions to {}.",
        target.path.display()
    );
    if let Some(rc) = activated {
        eprintln!(
            "Updated {} so the completion directory is on the search path.",
            rc.display()
        );
    }
    eprintln!("Restart your shell to activate them.");

    Ok(())
}

/// Run the binary with `COMPLETE=<shell>` to obtain its dynamic completion script.
fn generated_script(shell: Shell) -> Result<String> {
    let exe = std::env::current_exe().context("could not determine the retoki executable path")?;
    let output = std::process::Command::new(&exe)
        .env("COMPLETE", shell.to_string())
        .output()
        .with_context(|| format!("failed to run {}", exe.display()))?;
    if !output.status.success() {
        anyhow::bail!(
            "{} exited with {} while generating {shell} completions",
            exe.display(),
            output.status
        );
    }
    String::from_utf8(output.stdout)
        .with_context(|| format!("{shell} completion script was not valid UTF-8"))
}

/// Append `line` to `path` unless it is already present. Returns whether the line
/// was newly added.
fn append_line_if_missing(path: &Path, line: &str) -> Result<bool> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let existing = fs::read_to_string(path).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == line) {
        return Ok(false);
    }

    let separator = if existing.is_empty() || existing.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    writeln!(file, "{separator}{line}")
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(true)
}

fn xdg_config_home() -> Result<PathBuf> {
    match std::env::var_os("XDG_CONFIG_HOME") {
        Some(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
        _ => Ok(home_dir()?.join(".config")),
    }
}

fn xdg_data_home() -> Result<PathBuf> {
    match std::env::var_os("XDG_DATA_HOME") {
        Some(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
        _ => Ok(home_dir()?.join(".local/share")),
    }
}

fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .context("$HOME is not set")
}

/// Arguments that are shared by all commands
#[derive(Clone, Debug, PartialEq, Eq, Args)]
pub struct ProfileArgs {
    /// Path to the profile to use
    #[clap(long, env = "RETOKI_PROFILE_PATH", default_value_os_t = default_profile_path(), add = ArgValueCandidates::new(crate::completion::profile_candidates))]
    pub profile: PathBuf,
}

fn default_profile_path() -> PathBuf {
    PathBuf::from("retoki-profiles/gitlab.yml")
}
