use std::{
    fs,
    io::Write,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};

pub trait VerifiedState {}
pub struct Verified;
impl VerifiedState for Verified {}
pub struct UnVerified;
impl VerifiedState for UnVerified {}

const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config<State: VerifiedState> {
    // TODO: add a option to check if we start sma with a config but it is 
    // already running, should start the same config again and start all the 
    // applications, or should we exit with out starting them, or should we 
    // start them and then send the info to the first running sma, so it 
    // handles the new pids that spawned.

    // TODO: add a option to start some starts with a console or not.

    // This is the version of this config file.
    version: u32,
    // This is where we the current working directory will be for the
    // applications that are started.
    cwd: Option<PathBuf>,
    // This is if we should also kill all the still living child processes of
    // the processes we spawned.
    cascade_kill: bool,
    // This is the applications that we are going to spawn.
    start: Vec<String>,
    // This is the index to the start list of applications that we spawned,
    // that we waiting for to exit and then we kill everything else we spawned.
    exit_on: Option<u8>,
    #[serde(skip)]
    _marker: PhantomData<State>,
}

impl Default for Config<UnVerified> {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            cwd: Some(PathBuf::default().clean()),
            cascade_kill: false,
            start: Default::default(),
            exit_on: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl<State: VerifiedState> Config<State> {
    pub fn get_version(&self) -> u32 {
        self.version
    }

    pub fn get_cwd(&self) -> Option<&Path> {
        self.cwd.as_deref()
    }

    pub fn get_cascade_kill(&self) -> bool {
        self.cascade_kill
    }

    pub fn get_start(&self) -> &[String] {
        self.start.as_slice()
    }

    pub fn get_exit_on(&self) -> Option<u8> {
        self.exit_on
    }
}

impl Config<Verified> {
    pub fn create_file(&self, file_path: PathBuf, force_overide: bool) -> anyhow::Result<()> {
        let mut file_options = fs::OpenOptions::new();

        if force_overide {
            file_options.create(true).truncate(true)
        } else {
            match file_path.try_exists() {
                Ok(true) => {
                    bail!(
                        "The config file `{}` already exist.\nUse flag `-f`, `--force-overide` to force a override of the config file.", 
                        file_path.display()
                    )
                }
                Ok(false) => file_options.create_new(true),
                Err(e) => {
                    bail!(anyhow!(e).context(anyhow!(
                        "Got an error while trying to see if `{}` exists.",
                        file_path.display()
                    )))
                }
            }
        }
        .write(true);

        let mut file = file_options.open(&file_path).or_else(|e| {
            bail!(anyhow!(e).context(anyhow!("Could not open `{}`.", file_path.display())))
        })?;
        file.write_all(&serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}

impl Config<UnVerified> {
    pub fn from_existing_config_file(file_path: PathBuf) -> anyhow::Result<Config<UnVerified>> {
        match file_path.try_exists() {
            Ok(true) => (),
            Ok(false) => bail!("Config file does not exist at `{}`.", file_path.display()),
            Err(e) => {
                bail!(anyhow!(e).context(anyhow!(
                    "Got an error while trying to see if `{}` exists.",
                    file_path.display()
                )))
            }
        }

        if !file_path.is_file() {
            bail!(
                "The given config file `{}` is not a json file.",
                file_path.display()
            )
        }

        let json = fs::read(&file_path)?;
        let mut config: Config<UnVerified> = serde_json::from_slice(&json).or_else(|e| {
            bail!(anyhow!(e).context(anyhow!(
                "Something went wrong when reading `{}`.",
                file_path.display()
            )))
        })?;

        if let Some(cwd) = &config.cwd {
            if let Some(file_dir) = file_path.parent() {
                config.cwd = Some(file_dir.join(cwd).clean());
            }
        }

        Ok(config)
    }

    pub fn new_config_to_file(file_path: PathBuf, force_overide: bool) -> anyhow::Result<()> {
        Config::default()
            .verify()?
            .create_file(file_path, force_overide)
    }

    pub fn new(start: Vec<String>, exit_on: Option<u8>) -> anyhow::Result<Config<Verified>> {
        Config {
            version: CONFIG_VERSION,
            cwd: None,
            cascade_kill: false,
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
            cascade_kill: self.cascade_kill,
            start: self.start,
            exit_on: self.exit_on,
            _marker: PhantomData::default(),
        })
    }

    fn validate_cwd(&self) -> anyhow::Result<()> {
        if let Some(cwd) = &self.cwd {
            if !cwd.is_dir() {
                bail!(
                    "The given current working directory (cwd) `{}` is not a directory",
                    cwd.display()
                )
            }
        }
        Ok(())
    }

    fn validate_start(&self) -> anyhow::Result<()> {
        // TODO: Check if the given command actually exist
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
