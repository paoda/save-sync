use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use thiserror::Error;

lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    PoisonedWriteError(#[from] PoisonError<RwLockWriteGuard<'static, Config>>),
    #[error(transparent)]
    PoisonedReadError(#[from] PoisonError<RwLockReadGuard<'static, Config>>),
    #[error("Failed to Deserialize save-sync configuration from disk.")]
    DeserializationError(#[from] toml::de::Error),
    #[error("Falied to Serialize save-sync configuration to disk.")]
    SerializationError(#[from] toml::ser::Error),
    #[error("{0} was found to be an invalid path.")]
    InvalidPath(String),
    #[error("{0} is not a valid UTF-8 compatible path")]
    IllegalPath(String),
    #[error("Unable to determine the file / path name of {0}")]
    UnknownFileName(String),
    #[error("Unable to determine the parent of {0}")]
    UnknownPathParent(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Config {
    pub db_location: PathBuf,
    pub data_location: PathBuf,
    pub xxhash_seed: i64, // Issue: https://github.com/alexcrichton/toml-rs/issues/256 (should be u64)
    pub local_username: String,
}

impl Default for Config {
    fn default() -> Config {
        let data_location = Self::get_default_data_path();
        let db_location = data_location.join("saves.db");

        Config {
            db_location,
            data_location,
            xxhash_seed: 1_912_251_925_143,
            local_username: "Default".to_string(),
        }
    }
}

impl<'a> Config {
    pub fn update(config: Config) -> Result<(), ConfigError> {
        let mut w = CONFIG.write()?;
        *w = config;

        Ok(())
    }

    pub fn static_config() -> Result<RwLockReadGuard<'a, Config>, ConfigError> {
        Ok(CONFIG.read()?)
    }

    pub fn clone_config() -> Result<Config, ConfigError> {
        Ok((*CONFIG.read()?).clone())
    }

    fn get_default_data_path() -> PathBuf {
        match ProjectDirs::from("moe", "paoda", "save-sync") {
            Some(project) => project.data_dir().to_path_buf(),
            None => {
                panic!("No valid home directory could be determined from the Operating System.")
            }
        }
    }
}

#[derive(Debug)]
pub struct ConfigManager {
    config_file_path: PathBuf,
}

impl Default for ConfigManager {
    fn default() -> Self {
        let path: PathBuf;

        // Look in the environment variable, and if nothing
        // is there then we use directories-rs
        match std::env::var("SAVE_SYNC_CONFIG_PATH") {
            Ok(env) => path = PathBuf::from(env),
            Err(_err) => {
                let base = ConfigManager::get_config_dir();
                path = base.join("settings.toml");
            }
        }

        ConfigManager {
            config_file_path: path,
        }
    }
}

impl ConfigManager {
    pub fn new<P: AsRef<Path>>(path: &P) -> ConfigManager {
        Self::create_config_directory(&path).expect("Unable to create save-sync config directory.");

        ConfigManager {
            config_file_path: path.as_ref().to_owned(),
        }
    }

    fn create_config_directory<P: AsRef<Path>>(path: &P) -> Result<(), ConfigError> {
        let parent = path.as_ref().parent().ok_or_else(|| {
            ConfigError::UnknownPathParent(path.as_ref().to_string_lossy().to_string())
        })?;

        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }

        Ok(Self::create_config_file(path)?)
    }

    fn create_config_file<P: AsRef<Path>>(path: &P) -> Result<(), ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            let config = Config::default();

            let toml_string = toml::to_string(&config)?;
            let mut file = File::create(path)?;
            file.write_all(toml_string.as_bytes())?;
        } else {
            let file = File::open(path)?;
            Self::update_config_from_file(&file)?;
        }

        Ok(())
    }

    fn update_config_from_file(file: &File) -> Result<(), ConfigError> {
        let mut buf_reader = BufReader::new(file);
        let mut toml_buf = vec![];
        buf_reader.read_to_end(&mut toml_buf)?;

        let config: Config = toml::from_slice(&toml_buf)?;
        Config::update(config)?;

        Ok(())
    }

    pub fn load_from_file(&self) -> Result<(), ConfigError> {
        let file = File::open(&self.config_file_path)?;
        Self::update_config_from_file(&file)?;

        Ok(())
    }

    pub fn write_to_file(&self) -> Result<(), ConfigError> {
        let config = Config::static_config()?;
        let toml_string = toml::to_string(&(*config))?;

        let mut file = File::create(&self.config_file_path)?;

        file.write_all(toml_string.as_bytes())?;

        Ok(())
    }

    pub fn get_config_dir() -> PathBuf {
        match ProjectDirs::from("moe", "paoda", "save-sync") {
            Some(project) => project.config_dir().to_path_buf(),
            None => panic!("No Valid home directory could be retrieved from the Operating System."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn config_update_valid_input() {
        let expected_data_location = PathBuf::from("new_data_location");
        let expected_xxhash_seed: i64 = rand::random();
        let expected_db_location = PathBuf::from("new_db_location");

        let expected = Config {
            data_location: expected_data_location.clone(),
            xxhash_seed: expected_xxhash_seed,
            db_location: expected_db_location.clone(),
            local_username: "SomeUser".to_string(),
        };

        Config::update(expected.clone()).unwrap();

        let actual = &*Config::static_config().unwrap();

        assert_eq!(*actual, expected);
    }

    #[test]
    pub fn verify_write_to_file() {
        let test_dir = TempDir::new().unwrap();
        let tmp_dir = test_dir.path();

        let settings_path: PathBuf = [tmp_dir, &PathBuf::from("settings.toml")].iter().collect();

        let expected_data_location = PathBuf::from("new_data_location");
        let expected_xxhash_seed: i64 = rand::random();
        let expected_db_location = PathBuf::from("new_db_location");

        let expected = Config {
            data_location: expected_data_location,
            xxhash_seed: expected_xxhash_seed,
            db_location: expected_db_location,
            local_username: "User1".to_string(),
        };

        let manager = ConfigManager::new(&settings_path);
        Config::update(expected.clone()).unwrap();
        manager.write_to_file().unwrap();

        let mut file = File::open(settings_path).unwrap();

        let mut toml_buf = vec![];
        file.read_to_end(&mut toml_buf).unwrap();

        let actual: Config = toml::from_slice(&toml_buf).unwrap();

        test_dir.close().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn verify_load_from_file() {
        // FIXME: This test sometimes fails. Why?
        let test_dir = TempDir::new().unwrap();
        let tmp_dir = test_dir.path();

        let settings_path: PathBuf = [tmp_dir, &PathBuf::from("settings.toml")].iter().collect();
        let manager = ConfigManager::new(&settings_path);
        let mut settings = File::create(&settings_path).unwrap();

        let expected_data_location = PathBuf::from("new_data_location");
        let expected_xxhash_seed: i64 = rand::random();
        let expected_db_location = PathBuf::from("new_db_location");

        let expected = Config {
            data_location: expected_data_location,
            xxhash_seed: expected_xxhash_seed,
            db_location: expected_db_location,
            local_username: "Default".to_string(),
        };

        let toml_str = toml::to_string(&expected).unwrap();
        settings.write_all(&toml_str.into_bytes()).unwrap();

        manager.load_from_file().unwrap();

        let actual = &*Config::static_config().unwrap();

        test_dir.close().unwrap();
        assert_eq!(*actual, expected);
    }

    #[test]
    fn verify_create_config_file() {
        let test_dir = TempDir::new().unwrap();
        let tmp_dir = test_dir.path();
        let expected = Config::default();
        let settings_path: PathBuf = [tmp_dir, &PathBuf::from("settings.toml")].iter().collect();

        ConfigManager::create_config_file(&settings_path).unwrap();
        let mut file = File::open(&settings_path).unwrap();
        let mut toml_string = String::new();

        file.read_to_string(&mut toml_string).unwrap();
        let actual: Config = toml::from_str(&toml_string).unwrap();

        test_dir.close().unwrap();
        assert_eq!(actual, expected);
    }
}
