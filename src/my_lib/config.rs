use std::{fs, io::Write, marker::PhantomData, path::PathBuf, process::exit};

use anyhow::{anyhow, bail};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};

use crate::my_lib::{
    cli::{Cli, Commands},
    errors::{ConfigFileError, CreateConfigError},
};

pub trait VerifiedState {}
pub struct Verified;
impl VerifiedState for Verified {}
pub struct UnVerified;
impl VerifiedState for UnVerified {}

const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config<State: VerifiedState> {
    pub version: u32,
    pub cwd: Option<PathBuf>,
    pub start: Vec<String>,
    pub exit_on: Option<u8>,
    #[serde(skip)]
    _marker: PhantomData<State>,
}

impl Default for Config<UnVerified> {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            cwd: Some(PathBuf::default().clean()),
            start: Default::default(),
            exit_on: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl Config<UnVerified> {
    pub fn from_cli(cli: Cli) -> anyhow::Result<Config<Verified>> {
        let config = match cli.command {
            Commands::Start { start, exit_on } => Config::new(start, exit_on),
            Commands::Config { file_path } => {
                // Does path exist check
                match file_path.try_exists() {
                    Ok(true) => (),
                    Ok(false) => bail!(ConfigFileError::ConfigNotExist(file_path)),
                    Err(e) => {
                        bail!(anyhow!(e).context(anyhow!(ConfigFileError::ConfigFinding(file_path))))
                    }
                }

                if !file_path.is_file() {
                    bail!(ConfigFileError::ConfigIsNotFile(file_path))
                }

                let json = fs::read(&file_path)?;
                let mut config: Config<UnVerified> =
                    serde_json::from_slice(&json).or_else(|e| {
                        bail!(anyhow!(e)
                            .context(anyhow!(ConfigFileError::ConfigParsing(file_path.clone()))))
                    })?;

                if let Some(cwd) = &config.cwd {
                    if let Some(file_dir) = file_path.parent() {
                        config.cwd = Some(file_dir.join(cwd).clean());
                    }
                }

                config.verify()
            }
            Commands::CreateConfig {
                file_path,
                force_overide,
            } => {
                let mut file_options = fs::OpenOptions::new();

                if force_overide {
                    file_options.create(true).truncate(true)
                } else {
                    match file_path.try_exists() {
                        Ok(true) => bail!(CreateConfigError::ConfigFileAlreadyExist(file_path)),
                        Ok(false) => file_options.create_new(true),
                        Err(e) => {
                            bail!(anyhow!(e)
                                .context(anyhow!(ConfigFileError::ConfigFinding(file_path))))
                        }
                    }
                }
                .write(true);

                let mut file = file_options.open(&file_path).or_else(|e| {
                    bail!(
                        anyhow!(e).context(anyhow!(ConfigFileError::ConfigCouldNotOpen(file_path)))
                    )
                })?;

                file.write_all(&serde_json::to_vec_pretty(&Config::default())?)?;

                exit(0);
            }
        };
        config
    }

    pub fn new(start: Vec<String>, exit_on: Option<u8>) -> anyhow::Result<Config<Verified>> {
        Config {
            version: CONFIG_VERSION,
            cwd: None,
            start,
            exit_on,
            _marker: Default::default(),
        }
        .verify()
    }

    pub fn verify(self) -> anyhow::Result<Config<Verified>> {
        self.validate_start()?;
        self.validate_exit_on()?;
        self.validate_cwd()?;
        Ok(Config {
            version: self.version,
            cwd: self.cwd,
            start: self.start,
            exit_on: self.exit_on,
            _marker: PhantomData::default(),
        })
    }

    fn validate_cwd(&self) -> anyhow::Result<()> {
        if let Some(cwd) = &self.cwd {
            if !cwd.is_dir() {
                bail!(ConfigFileError::ConfigCwdIsNotADir(cwd.clone()))
            }
        }
        Ok(())
    }

    fn validate_start(&self) -> anyhow::Result<()> {
        if self.start.is_empty() {
            bail!(ConfigFileError::ConfigStartIsEmpty)
        }
        Ok(())
    }

    fn validate_exit_on(&self) -> anyhow::Result<()> {
        if let Some(index) = self.exit_on {
            if self.start.get(index as usize).is_none() {
                bail!({
                    let exit_on_name = std::env::args()
                        .rfind(|e| e == "-e" || e == "--exit-on")
                        .unwrap();
                    let mut msg = format!(
                        "The `{exit_on_name}` arg index `{index}` could not be found in `start`."
                    );
                    let len = self.start.len();
                    if index as usize - len == 0 {
                        let help_msg = &format!("\n\nHelp: The `{exit_on_name}` arg is `{index}` and length of `start` is {len}, but `start` is zero indexed. This means that to get the last element of start we use length - 1 as index. So in this case index {} is the last index.", index - 1);
                        msg += help_msg;
                    }
                    msg
                })
            }
        }
        Ok(())
    }
}
