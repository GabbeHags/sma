use std::{
    fs,
    io::Write,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail};

use path_clean::PathClean;
use serde::{Deserialize, Serialize};

pub trait VerifiedState: private::VerifiedStatePrivate {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Verified;
impl VerifiedState for Verified {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnVerified;
impl VerifiedState for UnVerified {}

mod private {
    pub trait VerifiedStatePrivate {}
    impl VerifiedStatePrivate for super::Verified {}
    impl VerifiedStatePrivate for super::UnVerified {}
}

const CONFIG_VERSION: u32 = 1;

#[cfg_attr(test, derive(Clone))]
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
    // This is the file path to the config file if it exists.
    #[serde(skip)]
    config_file_path: Option<PathBuf>,
    #[serde(skip)]
    _marker: PhantomData<State>,
}

impl Default for Config<UnVerified> {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            cwd: None,
            cascade_kill: false,
            start: Default::default(),
            exit_on: Default::default(),
            config_file_path: None,
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

    pub fn get_config_file_path(&self) -> Option<&Path> {
        self.config_file_path.as_deref()
    }
}

impl Config<Verified> {
    pub fn create_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        force_overide: bool,
    ) -> anyhow::Result<()> {
        let mut file_options = fs::OpenOptions::new();
        let file_path = file_path.as_ref();
        if force_overide {
            file_options.create(true).truncate(true)
        } else {
            match file_path.try_exists() {
                Ok(true) => {
                    bail!(
                        // TODO: remove this error msg, because this is not something the config cares about, this should be delegated to the caller.
                        // "The config file `{}` already exist.\nUse flag `-f`, `--force-overide` to force a override of the config file."
                        "The config file you are trying to create `{}` already exist.",
                        file_path.display()
                    )
                }
                Ok(false) => file_options.create_new(true),
                Err(e) => {
                    bail!(anyhow!(e).context(anyhow!(
                        "Got an error while trying to check if `{}` exists.",
                        file_path.display()
                    )))
                }
            }
        }
        .write(true);

        let mut file = file_options.open(file_path).or_else(|e| {
            bail!(anyhow!(e).context(anyhow!("Could not open `{}`.", file_path.display())))
        })?;
        file.write_all(&serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}

impl Config<UnVerified> {
    pub fn from_existing_config_file<P: AsRef<Path>>(
        file_path: P,
    ) -> anyhow::Result<Config<UnVerified>> {
        let file_path = file_path.as_ref();
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
                "The given path `{}` to the config, is not a valid file.",
                file_path.display()
            )
        }

        let mut config: Config<UnVerified> =
            serde_json::from_str(fs::read_to_string(file_path)?.as_str()).or_else(|e| {
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

        config.config_file_path = Some(file_path.to_path_buf());

        Ok(config)
    }

    pub fn new_config_to_file<P: AsRef<Path>>(
        file_path: P,
        force_overide: bool,
    ) -> anyhow::Result<()> {
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
            config_file_path: None,
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
            config_file_path: self.config_file_path,
            _marker: Default::default(),
        })
    }

    fn validate_cwd(&self) -> anyhow::Result<()> {
        // checks so the given cwd is an existing directory
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
        // checks if exit_on is given then, its index must exist
        if let Some(index) = self.exit_on {
            if self.start.is_empty() {
                bail!("exitOn should not be specified if start is empty.")
            }
            if self.start.get(index as usize).is_none() {
                bail!({
                    let mut msg =
                        format!("The `exitOn` arg index `{index}` could not be found in `start`.");
                    let len = self.start.len();
                    if index as usize - len == 0 {
                        let help_msg = &format!("\n\nHelp: The `exitOn` arg is `{index}` and length of `start` is {len}, but `start` is zero indexed. This means that to get the last element of start we use length - 1 as index. So in this case index {} is the last index.", index - 1);
                        msg += help_msg;
                    }
                    msg
                })
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests_config_version_1 {
    use std::str::FromStr;

    use super::*;
    use tempdir::TempDir;

    impl<MyState: VerifiedState, OtherState: VerifiedState> PartialEq<Config<OtherState>>
        for Config<MyState>
    {
        fn eq(&self, other: &Config<OtherState>) -> bool {
            self.version == other.version
                && self.cwd == other.cwd
                && self.cascade_kill == other.cascade_kill
                && self.start == other.start
                && self.exit_on == other.exit_on
        }
    }

    #[test]
    fn test_new_config_to_file_overide_true() {
        let temp_dir = TempDir::new("sma_config_test").unwrap();
        let config_name = temp_dir.path().join("test_config.json");
        Config::new_config_to_file(&config_name, false).unwrap();
        Config::new_config_to_file(&config_name, true).unwrap();

        assert_eq!(
            Config::default(),
            Config::from_existing_config_file(&config_name).unwrap()
        )
    }

    #[test]
    fn test_new_config_to_file_overide_false() {
        let temp_dir = TempDir::new("sma_config_test").unwrap();
        let config_name = temp_dir.path().join("test_config.json");
        Config::new_config_to_file(&config_name, false).unwrap();

        assert_eq!(
            Config::default(),
            Config::from_existing_config_file(&config_name).unwrap()
        )
    }

    #[test]
    fn test_from_existing_config_file_err_file_does_not_exist() {
        let name = "does_not_exist.json";
        assert_eq!(
            format!("Config file does not exist at `{}`.", name),
            Config::from_existing_config_file(PathBuf::from_str(name).unwrap())
                .unwrap_err()
                .to_string()
        )
    }

    #[test]
    fn test_from_existing_config_file_err_not_json_file() {
        let temp_dir = TempDir::new("sma_config_test").unwrap();
        assert_eq!(
            format!(
                "The given path `{}` to the config, is not a valid file.",
                temp_dir.path().display()
            ),
            Config::from_existing_config_file(temp_dir.path())
                .unwrap_err()
                .to_string()
        )
    }

    #[test]
    fn test_from_existing_config_file_err_something_went_wrong() {
        let temp_dir = TempDir::new("sma_config_test").unwrap();
        let config_name = temp_dir.path().join("test_config");
        fs::File::create(&config_name).unwrap();
        assert_eq!(
            format!(
                "Something went wrong when reading `{}`.",
                config_name.display()
            ),
            Config::from_existing_config_file(&config_name)
                .unwrap_err()
                .to_string()
        )
    }

    #[test]
    fn test_from_existing_config_file_default_1() {
        let temp_dir = TempDir::new("sma_config_test").unwrap();
        let config_name = temp_dir.path().join("test_config.json");
        Config::default()
            .verify()
            .unwrap()
            .create_file(&config_name, false)
            .unwrap();

        assert_eq!(
            Config::default(),
            Config::from_existing_config_file(&config_name).unwrap()
        )
    }

    #[test]
    fn test_from_existing_config_file_default_2() {
        let temp_dir = TempDir::new("sma_config_test").unwrap();
        let config_name = temp_dir.path().join("test_config.json");
        Config::new_config_to_file(&config_name, false).unwrap();

        assert_eq!(
            Config::default(),
            Config::from_existing_config_file(&config_name).unwrap()
        )
    }

    #[test]
    fn test_from_existing_config_file_with_cwd() {
        let temp_dir = TempDir::new("sma_config_test").unwrap();
        let test_dir_name = temp_dir.path().join("test_dir");
        fs::create_dir(&test_dir_name).unwrap();
        let config_name = temp_dir.path().join("test_config.json");
        let config = Config {
            cwd: Some(test_dir_name.clone()),
            ..Default::default()
        };

        config
            .clone()
            .verify()
            .unwrap()
            .create_file(&config_name, false)
            .unwrap();

        assert_eq!(
            config,
            Config::from_existing_config_file(&config_name).unwrap()
        )
    }

    #[test]
    fn test_verify_eq() {
        let config_unverified = Config::default();
        let config_verified = Config::default().verify().unwrap();
        assert_eq!(config_unverified, config_verified);
    }

    #[test]
    fn test_default_config() {
        let config_default = Config::default();
        let config: Config<UnVerified> = Config {
            version: 1,
            cwd: None,
            cascade_kill: false,
            start: vec![],
            exit_on: None,
            config_file_path: None,
            _marker: Default::default(),
        };
        assert_eq!(config, config_default)
    }

    #[test]
    fn test_validate_cwd_ok() {
        let config = Config {
            cwd: Some(".".into()),
            ..Default::default()
        };
        config.validate_cwd().unwrap()
    }

    #[test]
    fn test_validate_cwd_err() {
        let none_existing_dir = "./does_not_exist";
        let config = Config {
            cwd: Some(none_existing_dir.into()),
            ..Default::default()
        };

        assert_eq!(
            format!(
                "The given current working directory (cwd) `{}` is not a directory",
                none_existing_dir
            ),
            config.validate_cwd().unwrap_err().to_string()
        )
    }

    #[test]
    fn test_validate_exit_on_ok() {
        assert!(Config::default().validate_exit_on().is_ok());
        assert!(Config {
            exit_on: Some(0),
            start: vec!["this_exists".into()],
            ..Default::default()
        }
        .validate_exit_on()
        .is_ok());
    }

    #[test]
    fn test_validate_exit_on_err() {
        let exit_on = 100;
        let config = Config {
            start: vec!["this_is_a_program".into()],
            exit_on: Some(exit_on),
            ..Default::default()
        };

        assert_eq!(
            format!("The `exitOn` arg index `{exit_on}` could not be found in `start`."),
            config.validate_exit_on().unwrap_err().to_string()
        )
    }

    #[test]
    fn test_validate_exit_on_err_empty_start() {
        let exit_on = 100;
        let config = Config {
            exit_on: Some(exit_on),
            ..Default::default()
        };

        assert_eq!(
            format!("exitOn should not be specified if start is empty."),
            config.validate_exit_on().unwrap_err().to_string()
        )
    }

    #[test]
    fn test_validate_exit_on_err_of_by_one() {
        let exit_on = 1;
        let config = Config {
            start: vec!["this_is_a_program".into()],
            exit_on: Some(exit_on),
            ..Default::default()
        };

        assert_eq!(
            format!("The `exitOn` arg index `{exit_on}` could not be found in `start`.\n\nHelp: The `exitOn` arg is `{exit_on}` and length of `start` is {exit_on}, but `start` is zero indexed. This means that to get the last element of start we use length - 1 as index. So in this case index {} is the last index.", exit_on-1),
            config.validate_exit_on().unwrap_err().to_string()
        )
    }
}
