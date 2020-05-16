use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use std::sync::{RwLock, RwLockReadGuard};

lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
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
        let db_location: PathBuf = [&data_location, &PathBuf::from("saves.db")]
            .iter()
            .collect();

        Config {
            db_location,
            data_location,
            xxhash_seed: 1_912_251_925_143,
            local_username: "Default".to_string(),
        }
    }
}

impl Config {
    pub fn update(config: Config) {
        let mut w = CONFIG.write().unwrap();
        *w = config;
    }

    pub fn static_config() -> RwLockReadGuard<'static, Config> {
        CONFIG.read().unwrap()
    }

    pub fn clone_config() -> Config {
        (*CONFIG.read().unwrap()).clone()
    }

    fn get_default_data_path() -> PathBuf {
        match ProjectDirs::from("moe", "paoda", "save-sync") {
            Some(project) => project.data_dir().to_path_buf(),
            None => panic!("No valid home directory could be retrieved from the Operating System."),
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
    pub fn new(path: PathBuf) -> ConfigManager {
        Self::create_config_directory(&path);

        ConfigManager {
            config_file_path: path,
        }
    }

    fn create_config_directory(path: &PathBuf) {
        let parent = path.parent().unwrap_or_else(|| {
            panic!(
                "Unable to determine parent directory of {}",
                path.to_string_lossy()
            )
        }); // TODO: Instead of panicking , handle this option as if it were a ConfigError

        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }

        Self::create_config_file(path);
    }

    fn create_config_file(path: &PathBuf) {
        if !path.exists() {
            let config = Config::default();

            let toml_string = toml::to_string(&config).unwrap();
            let mut file = File::create(path).unwrap();
            file.write_all(toml_string.as_bytes()).unwrap();
        } else {
            // FIXME: Code Duplication here :\

            let file = File::open(path).unwrap();
            Self::update_config_from_file(&file);
        }
    }

    fn update_config_from_file(file: &File) {
        let mut buf_reader = BufReader::new(file);
        let mut toml_buf = vec![];
        buf_reader.read_to_end(&mut toml_buf).unwrap();

        let config: Config = toml::from_slice(&toml_buf).unwrap();
        Config::update(config);
    }

    pub fn load_from_file(&self) {
        let file = File::open(&self.config_file_path).unwrap();
        Self::update_config_from_file(&file);
    }

    pub fn write_to_file(&self) {
        let config = Config::static_config();
        let toml_string = toml::to_string(&(*config)).unwrap();

        let mut file = File::create(&self.config_file_path).unwrap();

        file.write_all(toml_string.as_bytes()).unwrap();
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

        Config::update(expected.clone());

        let actual = &*Config::static_config();

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

        Config::update(expected.clone());
        let manager = ConfigManager::new(settings_path.clone());
        manager.write_to_file();

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
        let manager = ConfigManager::new(settings_path.clone());
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

        manager.load_from_file();

        let actual = &*Config::static_config();

        test_dir.close().unwrap();
        assert_eq!(*actual, expected);
    }

    #[test]
    fn verify_create_config_file() {
        let test_dir = TempDir::new().unwrap();
        let tmp_dir = test_dir.path();
        let expected = Config::default();
        let settings_path: PathBuf = [tmp_dir, &PathBuf::from("settings.toml")].iter().collect();

        ConfigManager::create_config_file(&settings_path);
        let mut file = File::open(&settings_path).unwrap();
        let mut toml_string = String::new();

        file.read_to_string(&mut toml_string).unwrap();
        let actual: Config = toml::from_str(&toml_string).unwrap();

        test_dir.close().unwrap();
        assert_eq!(actual, expected);
    }
}
