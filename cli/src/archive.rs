use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use options::*;
use save_sync::archive::query::SaveQuery;
use save_sync::config::Config;
use save_sync::models::{NewFile, NewSave, Save, User};
use save_sync::Archive as BaseArchive;
use save_sync::Database;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub struct Archive {}

impl Archive {
    pub fn create_save(db: &Database, user: &User, path: &PathBuf, opt: SaveOptions) -> Result<()> {
        if !path.exists() {
            let path_str = path.to_string_lossy();
            let err = anyhow!("{} does not exist on disk.", path_str);
            return Err(err);
        }

        let time = Utc::now().naive_utc();
        let mut uuid_buf = Uuid::encode_buffer();
        let uuid = Uuid::new_v4().to_hyphenated().encode_lower(&mut uuid_buf);
        let backup_pathbuf = Self::create_backup_path(path, &uuid)?;
        let backup_path = backup_pathbuf.to_str().with_context(|| {
            let path_str = backup_pathbuf.to_string_lossy();
            format!("The backup path \"{}\" was not UTF-8 compliant.", path_str)
        })?;
        let save_path = path.to_str().with_context(|| {
            format!("{} is not a UTF-8 compliant path.", path.to_string_lossy())
        })?;
        let friendly_name = {
            match opt.friendly_name {
                Some(name) => name,
                None => "",
            }
        };

        let new_save = NewSave {
            friendly_name,
            save_path,
            backup_path,
            uuid,
            user_id: user.id,
            created_at: time,
            modified_at: time,
        };

        db.create_save(new_save);
        let query = SaveQuery::new().with_uuid(uuid);
        let save = db.get_save(query).with_context(|| {
            let path_str = new_save.save_path;
            format!("Unable to query {} from db.", path_str)
        })?;

        let files = Self::crawl(path);
        for file in files {
            Self::create_file(db, &save, &file)?;
        }

        Ok(())
    }

    fn create_file(db: &Database, save: &Save, path: &PathBuf) -> Result<()> {
        let file_path = path.to_str().with_context(|| {
            format!("{} is not a UTF-8 compliant path.", path.to_string_lossy())
        })?;

        let time = Utc::now().naive_utc();
        let file_hash = &{
            let num = BaseArchive::calc_hash(path);
            BaseArchive::u64_to_byte_vec(num)
        };

        let new_file = NewFile {
            file_path,
            file_hash,
            save_id: save.id,
            created_at: time,
            modified_at: time,
        };

        db.create_file(new_file);
        Ok(())
    }

    fn create_backup_path(path: &PathBuf, uuid: &str) -> Result<PathBuf> {
        let config = Config::static_config();
        let root_path = &config.data_location;
        let name = path.file_name().with_context(|| {
            let path_str = path.to_string_lossy();
            format!("Unable to determine the name (last part) of {}", path_str)
        })?;

        let mut backup_path = PathBuf::new();
        backup_path.push(root_path);
        backup_path.push(uuid);
        backup_path.push(name);

        Ok(backup_path)
    }

    fn crawl(path: &PathBuf) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = vec![];
        let result = fs::read_dir(path);

        match result {
            Err(_) => files,
            Ok(list) => {
                let valid = list.map(|entry| entry.unwrap().path());
                for path in valid {
                    if path.is_dir() {
                        files.append(&mut Self::crawl(&path))
                    } else {
                        files.push(path)
                    }
                }
                files
            }
        }
    }
}

pub mod options {
    pub struct SaveOptions<'a> {
        pub friendly_name: Option<&'a str>,
    }
}
