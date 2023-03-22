use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum CreateConfigError {
    #[error("The config file `{}` already exist.", _0.display())]
    ConfigFileAlreadyExist(PathBuf),
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigFileError {
    #[error(
        "Config file does not have the correct extension. Expected json, but found `{found}`."
    )]
    ConfigWrongExtension { found: String },

    #[error("Config file does not have the correct extension. Expected json, but found a extension that contains none utf-8 symbols.")]
    ConfigNoneUtf8Extension,

    #[error(
        "Config file does not have the correct extension. Expected json, but found no extension."
    )]
    ConfigNoExtensionFound,

    #[error("Config file does not exist at `{}`.", _0.display())]
    ConfigNotExist(PathBuf),

    #[error("Something went wrong when reading `{}`.", _0.display())]
    ConfigParsing(PathBuf),

    #[error("The given config file is not a json file.")]
    ConfigIsNotFile(PathBuf),

    #[error("Could not open `{}`.", _0.display())]
    ConfigCouldNotOpen(PathBuf),

    #[error("No arguments where given to `start`.")]
    ConfigStartIsEmpty,

    #[error("Got an error while trying to see if `{}` exists.", _0.display())]
    ConfigFinding(PathBuf),

    #[error("The given current working directory (cwd) `{}` is not a directory", _0.display())]
    ConfigCwdIsNotADir(PathBuf),
}
