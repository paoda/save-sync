use anyhow::{anyhow, Context, Result};
use change::{SaveUpdate, Type};
use chrono::Utc;
use options::*;
use save_sync::archive::query::{FileQuery, SaveQuery};
use save_sync::config::Config;
use save_sync::models::{NewFile, NewSave, Save, User};
use save_sync::Archive as BaseArchive;
use save_sync::Database;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub struct Archive {}

impl Archive {
    pub fn create_save<P: AsRef<Path>>(
        db: &Database,
        user: &User,
        path: &P,
        opt: SaveOptions,
    ) -> Result<()> {
        if !path.as_ref().exists() {
            let path_str = path.as_ref().to_string_lossy();
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
        let save_path = path.as_ref().to_str().with_context(|| {
            format!(
                "{} is not a UTF-8 compliant path.",
                path.as_ref().to_string_lossy()
            )
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

        // After thinking for a couple minutes I have come to the conclusion that:
        // Having useless files in the backup folder is better than having a save in the db
        // which isn't actually backed up like we assume it to be
        // Therefore we copy files and only upon success do we actually write to db.
        let files = Self::crawl(path);
        Self::copy_save_files(&new_save, &files)?;

        db.create_save(new_save);
        let query = SaveQuery::new().with_uuid(uuid);
        let save = db.get_save(query).with_context(|| {
            let path_str = new_save.save_path;
            format!("Unable to query {} from db.", path_str)
        })?;

        for file in files {
            if file.is_file() {
                // FIXME: Empty Directories are on disk but not tracked in Database.
                Self::create_file(db, &save, &file)?;
            }
        }

        Ok(())
    }

    pub fn delete_save(db: &Database, save: &Save) -> Result<()> {
        // We'd rather have abandoned files than a save with missing backup files
        // Therefore we should delete the save first, and then files later.
        let backup_path = Path::new(&save.backup_path)
            .parent()
            .with_context(|| format!("Unable to determine parent of {}", save.backup_path))?;

        // Delete Related files in database first due to Database Constraints
        let files_query = FileQuery::new().with_save_id(save.id);
        let option = db.get_files(files_query);

        if let Some(files) = option {
            for file in files {
                let file_query = FileQuery::new().with_id(file.id);
                db.delete_file(file_query);
            }
        }

        let save_query = SaveQuery::new().with_id(save.id);
        db.delete_save(save_query);

        // Now Delete the Files on disk
        fs::remove_dir_all(backup_path)?;
        Ok(())
    }

    pub fn update_save(db: &Database, save: &Save) -> Result<Option<String>> {
        let changes = Self::check_save(db, save)?;
        let backup_path = Path::new(&save.backup_path);
        let mut changelog = String::new();

        if changes.is_empty() {
            return Ok(None);
        }

        for log in changes {
            let file_path = log.path;
            match log.change {
                Type::Missing => {
                    changelog.push_str(&format!(
                        "\nMissing (NOW DELETING!!!): {}",
                        file_path.to_string_lossy()
                    ));

                    //TODO: Be a bit more careful about deleting files
                    let query = FileQuery::new().with_path(&file_path);

                    db.delete_file(query);
                    let backup_path = Self::get_backup_path(&file_path, &backup_path)?;
                    fs::remove_file(backup_path)?;
                }
                Type::New => {
                    changelog.push_str(&format!("\nNew: {}", file_path.to_string_lossy()));

                    Self::copy_file_to_backup_dir(&backup_path, &file_path)?;
                    Self::create_file(db, save, &file_path)?;
                }
                Type::Update => {
                    changelog.push_str(&format!("\nUpdated: {}", file_path.to_string_lossy()));

                    Self::copy_file_to_backup_dir(&backup_path, &file_path)?;
                    Self::update_file(db, &file_path)?;
                }
            }
        }

        Ok(Some(changelog))
    }

    pub fn check_save(db: &Database, save: &Save) -> Result<Vec<SaveUpdate>> {
        use std::collections::HashMap;

        let mut result = vec![];
        let query = FileQuery::new().with_save_id(save.id);
        let tracked = db.get_files(query).with_context(|| {
            let path = &save.save_path;
            let name = &save.friendly_name;

            if name.is_empty() {
                format!("{} does not have any files associated with it.", path)
            } else {
                format!("{} does not have any files associated with it.", name)
            }
        })?;

        let path = Path::new(&save.save_path);
        let current = Self::crawl(&path);

        // Check For Missing & Build
        let mut tracked_hash_map = HashMap::new();

        for file in tracked {
            // While we're at it, build a HashMap
            // FIXME: Can we do this with less allocations?
            tracked_hash_map.insert(file.file_path.clone(), file.file_hash.clone());

            // if current tracked file does not match any on disk
            if !current.iter().any(|path| file == *path) {
                result.push(SaveUpdate {
                    change: Type::Missing,
                    path: PathBuf::from(file.file_path),
                })
            }
        }

        for file_path in current {
            if file_path.is_file() {
                let file_str = file_path.to_str().context(format!(
                    "Unable to convert {} to a UTF-8 String",
                    file_path.to_string_lossy()
                ))?;

                match tracked_hash_map.get(file_str) {
                    Some(expected) => {
                        let actual = {
                            let num = BaseArchive::calc_hash(&file_path)?;
                            BaseArchive::u64_to_byte_vec(num)?
                        };

                        if actual != *expected {
                            result.push(SaveUpdate {
                                change: Type::Update,
                                path: file_path,
                            })
                        }
                    }
                    None => result.push(SaveUpdate {
                        change: Type::New,
                        path: file_path,
                    }),
                }
            }
        }

        Ok(result)
    }

    pub fn old_check_save(db: &Database, save: &Save) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        use std::collections::HashMap;

        let mut new_files: Vec<PathBuf> = vec![];
        let mut changed_files: Vec<PathBuf> = vec![];

        let query = FileQuery::new().with_save_id(save.id);
        let tracked_files = db.get_files(query).with_context(|| {
            if save.friendly_name.is_empty() {
                format!(
                    "Save with path \"{}\" does not have any files associated with it.",
                    save.save_path
                )
            } else {
                format!(
                    "{} does not have any files associated with it.",
                    save.friendly_name
                )
            }
        })?;

        let mut tracked_files_map: HashMap<String, Vec<u8>> = HashMap::new();

        for file in tracked_files {
            tracked_files_map.insert(file.file_path, file.file_hash);
        }

        let save_path = Path::new(&save.save_path);
        let current_save_files = Self::crawl(&save_path);

        for file_path in current_save_files {
            if file_path.is_file() {
                let string = file_path.to_str().with_context(|| {
                    let path_str = file_path.to_string_lossy();
                    format!("Unable to convert {} to a UTF-8 String", path_str)
                })?;

                match tracked_files_map.get(string) {
                    Some(expected) => {
                        let actual = {
                            let hash_num = BaseArchive::calc_hash(&file_path)?;
                            BaseArchive::u64_to_byte_vec(hash_num)?
                        };

                        if actual != *expected {
                            changed_files.push(file_path)
                        }
                    }
                    None => new_files.push(file_path),
                }
            }
        }

        Ok((new_files, changed_files))
    }

    fn create_file<P: AsRef<Path>>(db: &Database, save: &Save, path: &P) -> Result<()> {
        let file_path = path.as_ref().to_str().with_context(|| {
            format!(
                "{} is not a UTF-8 compliant path.",
                path.as_ref().to_string_lossy()
            )
        })?;

        let time = Utc::now().naive_utc();
        let file_hash = &{
            let num = BaseArchive::calc_hash(path)?;
            BaseArchive::u64_to_byte_vec(num)?
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

    fn update_file<P: AsRef<Path>>(db: &Database, path: &P) -> Result<()> {
        use save_sync::models::EditFile;

        let query = FileQuery::new().with_path(path);
        let time = Utc::now().naive_utc();
        let original_file = db.get_file(query).with_context(|| {
            let path_str = path.as_ref().to_string_lossy();
            format!(
                "Unable to retrieve file with path {} from the database.",
                path_str
            )
        })?;
        let file_hash = &{
            let hash_num = BaseArchive::calc_hash(path)?;
            BaseArchive::u64_to_byte_vec(hash_num)?
        };

        let edit = EditFile {
            id: original_file.id,
            file_hash,
            modified_at: time,
        };

        db.update_file(edit);

        Ok(())
    }

    fn create_backup_path<P: AsRef<Path>>(path: &P, uuid: &str) -> Result<PathBuf> {
        let config = Config::static_config();
        let root_path = &config.data_location;
        let name = path.as_ref().file_name().with_context(|| {
            let path_str = path.as_ref().to_string_lossy();
            format!("Unable to determine the name (last part) of {}", path_str)
        })?;

        let backup_path = Path::new(root_path).join(uuid).join(name);
        Ok(backup_path)
    }

    fn crawl<P: AsRef<Path>>(path: &P) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = vec![];
        let result = fs::read_dir(path);

        match result {
            Err(_) => files,
            Ok(list) => {
                let valid = list.map(|entry| entry.unwrap().path());
                for path in valid {
                    if path.is_dir() {
                        files.append(&mut Self::crawl(&path))
                    }
                    files.push(path) // If we just want files, we can filter later.
                }
                files
            }
        }
    }

    fn copy_save_files<P: AsRef<Path>>(save: &NewSave, files: &[P]) -> Result<()> {
        let backup_path = Path::new(save.backup_path);

        for file_path in files {
            Self::copy_file_to_backup_dir(&backup_path, &file_path.as_ref())?;
        }

        Ok(())
    }

    fn get_backup_path<P: AsRef<Path>, Q: AsRef<Path>>(
        file_path: &P,
        backup_path: &Q,
    ) -> Result<PathBuf> {
        let common_component_name = backup_path.as_ref().file_name().with_context(|| {
            let path_str = backup_path.as_ref().to_string_lossy();
            format!("Unable to determine file / directory name of {}", path_str)
        })?;

        let mut base = PathBuf::new();

        for component in file_path.as_ref().components() {
            base.push(component);

            if component.as_os_str() == common_component_name {
                break;
            }
        }

        let prefixless = file_path.as_ref().strip_prefix(base)?;
        Ok(backup_path.as_ref().join(prefixless))
    }

    fn copy_file_to_backup_dir<P: AsRef<Path>, Q: AsRef<Path>>(
        backup_path: &P,
        file_path: &Q,
    ) -> Result<()> {
        let backup_destination = Self::get_backup_path(file_path, backup_path)?;

        if file_path.as_ref().is_dir() {
            // We just want to make sure that directory exists and re-create it if it doesnt
            if !backup_destination.exists() {
                fs::create_dir_all(backup_destination)?;
            }
        } else {
            // I assume if it's not a directory it's a file
            let backup_destination_parent = backup_destination.parent().with_context(|| {
                let path_str = backup_destination.to_string_lossy();
                format!("Unable to determine parent of {}", path_str)
            })?;

            if !backup_destination_parent.exists() {
                // It's good to be on the safer side.
                fs::create_dir_all(backup_destination_parent)?;
            }

            fs::copy(file_path, backup_destination)?;
        }

        Ok(())
    }
}

pub mod change {
    use std::path::PathBuf;

    pub struct SaveUpdate {
        pub change: Type,
        pub path: PathBuf,
    }
    pub enum Type {
        Update,
        New,
        Missing,
    }
}

pub mod options {
    pub struct SaveOptions<'a> {
        pub friendly_name: Option<&'a str>,
    }
}
