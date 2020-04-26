use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use toml;

lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub db_location: PathBuf,
    pub data_location: PathBuf,
    pub xxhash_seed: u64,
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
        }
    }
}

impl Config {
    fn update(config: Config) {
        let mut w = CONFIG.write().unwrap();
        *w = config;
    }

    fn static_config() -> RwLockReadGuard<'static, Config> {
        CONFIG.read().unwrap()
    }

    fn clone_config() -> Config {
        (*CONFIG.read().unwrap()).clone()
    }

    fn get_default_data_path() -> PathBuf {
        match ProjectDirs::from("moe", "paoda", "Save Sync") {
            Some(project) => return project.data_dir().to_path_buf(),
            None => panic!("No valid home directory could be retrieved from the Operating System."),
        }
    }
}

#[derive(Default, Debug)]
pub struct ConfigManager {
    config_file_path: PathBuf,
}

impl ConfigManager {
    pub fn new(path: PathBuf) -> ConfigManager {
        Self::create_config_directory(&path);

        ConfigManager {
            config_file_path: path,
        }
    }

    fn create_config_directory(path: &PathBuf) {
        let parent = path.parent().expect(&format!(
            "Unable to determine parent directory of {}",
            path.to_string_lossy()
        )); // TODO: Instead of panicking , handle this option as if it were a ConfigError

        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }

        Self::create_config_file(path);
    }

    fn create_config_file(path: &PathBuf) {
        if !path.exists() {
            let config = Config::default();

            let toml_string = toml::to_string(&config).unwrap();
            let mut file = fs::File::create(path).unwrap();
            file.write_all(toml_string.as_bytes()).unwrap();
        } else {
            // FIXME: Code Duplication here :\

            let file = fs::File::open(path).unwrap();
            let mut buf_reader = BufReader::new(file);
            let mut toml_buffer = vec![];
            buf_reader.read_to_end(&mut toml_buffer).unwrap();

            let config = toml::from_slice::<Config>(&toml_buffer).unwrap();

            Config::update(config);
        }
    }

    pub fn load_from_file(&self) {
        let file = fs::File::open(&self.config_file_path).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut toml_buffer = vec![];
        buf_reader.read_to_end(&mut toml_buffer).unwrap();

        let config = toml::from_slice::<Config>(&toml_buffer).unwrap();

        Config::update(config)
    }

    pub fn write_to_file(&self) {
        let config = Config::static_config();
        let toml_string = toml::to_string(&(*config)).unwrap();

        let mut file = fs::File::create(&self.config_file_path).unwrap();

        file.write_all(toml_string.as_bytes()).unwrap();
    }

    pub fn get_config_dir() -> PathBuf {
        match ProjectDirs::from("moe", "paoda", "Save Sync") {
            Some(project) => return project.config_dir().to_path_buf(),
            None => panic!("No Valid home directory could be retrieved from the Operating System."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn config_update_valid_input() {
        let expected_data_location = PathBuf::from("new_data_location");
        let expected_xxhash_seed = 12345;
        let expected_db_location = PathBuf::from("new_db_location");

        let new_config = Config {
            data_location: expected_data_location.clone(),
            xxhash_seed: expected_xxhash_seed,
            db_location: expected_db_location.clone(),
        };

        Config::update(new_config);

        let config = &*Config::static_config();

        assert_eq!(config.data_location, expected_data_location);
        assert_eq!(config.xxhash_seed, expected_xxhash_seed);
        assert_eq!(config.db_location, expected_db_location);
    }
}
