use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

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

pub fn parse_args<Args, T>(args: Args) -> anyhow::Result<Commands>
where
    Args: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    Ok(Cli::try_parse_from(args)?.command)
    // Cli::parse().command
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

fn check_extension<P: AsRef<Path>>(file_path: P) -> anyhow::Result<()> {
    fn priv_check_extension(file_path: &Path) -> anyhow::Result<()> {
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
    // Extension checks
    priv_check_extension(file_path.as_ref())
}

#[cfg(test)]
mod tests_cli {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_get_args_create_config_path_bad_extension() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["create-config".into(), "apa.test".into()]);
        parse_args(args).unwrap_err();
    }

    #[test]
    fn test_get_args_create_config_path_no_extension() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["create-config".into(), "apa".into()]);
        parse_args(args).unwrap_err();
    }

    #[test]
    fn test_get_args_create_config_path_overide() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["create-config".into(), "apa.json".into(), "-f".into()]);
        parse_args(args).unwrap();
    }

    #[test]
    fn test_get_args_create_config_path() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["create-config".into(), "apa.json".into()]);
        parse_args(args).unwrap();
    }

    #[test]
    fn test_get_args_create_config() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["create-config".into()]);
        parse_args(args).unwrap();
    }

    #[test]
    fn test_get_args_config() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["config".into()]);
        parse_args(args).unwrap();
    }

    #[test]
    fn test_get_args_config_good_extension() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["config".into(), "test.json".into()]);
        parse_args(args).unwrap();
    }

    #[test]
    fn test_get_args_config_wrong_extension() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["config".into(), "test.apa".into()]);

        parse_args(args).unwrap_err();
    }

    #[test]
    fn test_get_args_config_no_extension() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["config".into(), "test".into()]);

        parse_args(args).unwrap_err();
    }

    #[test]
    fn test_get_args_start_not_exit() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["start".into(), "".into()]);
        parse_args(args).unwrap();
    }

    #[test]
    fn test_get_args_start_wtih_exit() {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.extend(["start".into(), "".into(), "0".into()]);
        parse_args(args).unwrap();
    }

    #[test]
    fn test_check_extension_no_extension() {
        let file_path = PathBuf::from_str("test_file").unwrap();
        let err_str = check_extension(file_path).unwrap_err().to_string();

        assert_eq!("Config file does not have the correct extension. Expected json, but found no extension.".to_string(), err_str);
    }

    #[test]
    fn test_check_extension_wrong_extension() {
        let file_path = PathBuf::from_str("test_file.exe").unwrap();
        let err_str = check_extension(file_path).unwrap_err().to_string();

        assert_eq!(
            "Config file does not have the correct extension. Expected json, but found `exe`."
                .to_string(),
            err_str
        );
    }

    #[test]
    fn test_cli_config_create_config_validator() {
        let file_path_str = "test_file.json";

        assert_eq!(
            std::env::current_dir().unwrap().join(file_path_str),
            cli_config_create_config_validator(file_path_str).unwrap()
        );
    }

    #[test]
    fn test_cli_config_file_path_validator() {
        let file_path_str = "test_file.json";

        assert_eq!(
            std::env::current_dir().unwrap().join(file_path_str),
            cli_config_create_config_validator(file_path_str).unwrap()
        );
    }
}
