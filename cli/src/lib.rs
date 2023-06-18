use std::path::{Path, PathBuf};

use anyhow::bail;
use clap::{Parser, Subcommand};
use path_clean::PathClean;

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Parser)]
#[command(name = "sma")]
#[command(author)]
#[command(version, propagate_version = true)]
#[command(help_template = "
----------------------------------------
Author: {author}
Version: {version}\n
{usage-heading} {usage}\n
{all-args} {tab}\n
----------------------------------------
")]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

pub fn get_args() -> Commands {
    Cli::parse().command
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Application to start.
    Start {
        /// Applications to start.
        #[arg(required = true, num_args = 1..)]
        start: Vec<String>,
        /// Specify an index which will be used to index in to `start`, to exit all
        /// started applications on if the application at that index stops running.
        #[arg(short, long)]
        exit_on: Option<u8>,
        // TODO: Add cwd.
        // TODO: Add cascade_kill.
    },

    /// Specify the config file for SMA.
    Config {
        /// The file path to the config file.
        #[arg(
            value_parser = cli_config_file_path_validator,
            default_value = CONFIG_FILE_NAME
        )]
        file_path: PathBuf,
    },

    /// Creates an empty config file.
    CreateConfig {
        /// The path to where the file should be created.
        #[arg(
            value_parser = cli_config_create_config_validator,
            default_value = CONFIG_FILE_NAME
        )]
        file_path: PathBuf,

        /// Flag to force a override of the config file.
        #[arg(short, long)]
        force_overide: bool,
    },
}

fn cli_config_file_path_validator(file_path: &str) -> anyhow::Result<PathBuf> {
    let file_path = std::env::current_dir()?.join(file_path).clean();

    // Extension checks
    check_extension(&file_path)?;

    Ok(file_path)
}

fn cli_config_create_config_validator(file_path: &str) -> anyhow::Result<PathBuf> {
    let file_path = std::env::current_dir()?.join(file_path).clean();

    // Extension checks
    check_extension(file_path.as_path())?;
    Ok(file_path)
}

fn check_extension(file_path: &Path) -> anyhow::Result<()> {
    // Extension checks
    match file_path.extension() {
        Some(ext) => {
            let ext = match ext.to_str() {
                Some(ext) => ext,
                None => bail!("Config file does not have the correct extension. Expected json, but found a extension that contains none utf-8 symbols."),
            };
            if ext != "json" {
                bail!("Config file does not have the correct extension. Expected json, but found `{}`.", ext.to_string());
            }
        }
        None => bail!("Config file does not have the correct extension. Expected json, but found no extension."),
    }
    Ok(())
}
