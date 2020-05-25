use crate::config::Config;
use chrono::prelude::{NaiveDateTime, Utc};
use std::fs::File;
use std::hash::Hasher;
use std::path::Path;
use tar::Archive as TarArchive;
use tar::Builder as TarBuilder;
use thiserror::Error;
use twox_hash::XxHash64;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("{0} was found to be an invalid path.")]
    InvalidPath(String),
    #[error("{0} is not a valid UTF-8 compatible path")]
    IllegalPath(String),
    #[error("Unable to determine the file / path name of {0}")]
    UnknownFileName(String),
    #[error("Unable to obtain reference to the global static config")]
    UnaccessableConfig,
}

#[derive(Debug, Default)]
pub struct Archive {}

impl Archive {
    pub fn new() -> Archive {
        Archive {}
    }

    pub fn u64_to_byte_vec(num: u64) -> Result<Vec<u8>, ArchiveError> {
        use byteorder::{LittleEndian, WriteBytesExt};

        let mut bytes: Vec<u8> = vec![];
        bytes.write_u64::<LittleEndian>(num)?;
        Ok(bytes)
    }

    pub fn calc_hash<P: AsRef<Path>>(path: &P) -> Result<u64, ArchiveError> {
        use std::io::Read;

        let path = path.as_ref();
        let config = Config::static_config().map_err(|_| ArchiveError::UnaccessableConfig)?;
        let seed = config.xxhash_seed as u64;

        // If hasher implements Writer we can use std::io::copy
        let mut hasher = XxHash64::with_seed(seed);
        let chunk_size = 0x4000;
        let mut file = File::open(path)?;

        loop {
            let mut chunk = Vec::with_capacity(chunk_size);
            let n = file
                .by_ref()
                .take(chunk_size as u64)
                .read_to_end(&mut chunk)?;

            if n == 0 {
                break;
            }
            hasher.write(&chunk);
        }
        Ok(hasher.finish())
    }

    pub fn compress_directory<P: AsRef<Path>, Q: AsRef<Path>>(
        source: &P,
        target: &Q,
    ) -> Result<(), ArchiveError> {
        let tar_file = File::create(target)?;
        let zstd_encoder = zstd::stream::Encoder::new(tar_file, 0)?;
        let mut archive = TarBuilder::new(zstd_encoder);

        let err = ArchiveError::UnknownFileName(source.as_ref().to_string_lossy().to_string());
        let base_name = source.as_ref().file_name().ok_or(err)?;

        let name = base_name
            .to_str()
            .ok_or_else(|| ArchiveError::IllegalPath(base_name.to_string_lossy().to_string()))?;

        archive.append_dir_all(name, source)?;
        let zstd_encoder = archive.into_inner()?;
        zstd_encoder.finish()?;
        Ok(())
    }

    pub fn compress_file<P: AsRef<Path>, Q: AsRef<Path>>(
        source: &P,
        target: &Q,
    ) -> Result<(), ArchiveError> {
        let mut file = File::open(source)?; // Reader
        let compressed_file = File::create(target)?; // Writer
        let mut zstd_encoder = zstd::stream::Encoder::new(compressed_file, 0)?;

        std::io::copy(&mut file, &mut zstd_encoder)?;
        zstd_encoder.finish()?;

        Ok(())
    }

    pub fn decompress_archive<P: AsRef<Path>, Q: AsRef<Path>>(
        source: &P,
        target: &Q,
    ) -> Result<(), ArchiveError> {
        let source_file = File::open(source)?;
        let zstd_decoder = zstd::stream::Decoder::new(source_file)?;
        let mut archive = TarArchive::new(zstd_decoder);

        Ok(archive.unpack(target)?)
    }

    pub fn decompress_file<P: AsRef<Path>, Q: AsRef<Path>>(
        source: &P,
        target: &Q,
    ) -> Result<(), ArchiveError> {
        let file = File::open(source)?;
        let mut target_file = File::create(target)?;

        Ok(zstd::stream::copy_decode(&file, &mut target_file)?)
    }

    /// Gets a unix time stamp in UTC±0:00
    pub fn get_utc_unix_time() -> NaiveDateTime {
        Utc::now().naive_utc()
    }
}

pub mod query {
    use std::path::Path;

    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct SaveQuery<'a> {
        pub id: Option<i32>,
        pub friendly_name: Option<&'a str>,
        pub uuid: Option<&'a str>,
        pub path: Option<&'a Path>,
        pub user_id: Option<i32>,
    }

    impl<'a> SaveQuery<'a> {
        pub fn new() -> SaveQuery<'a> {
            SaveQuery {
                id: None,
                friendly_name: None,
                uuid: None,
                path: None,
                user_id: None,
            }
        }
        pub fn with_id(mut self, id: i32) -> SaveQuery<'a> {
            self.id = Some(id);
            self
        }

        pub fn with_path<P: AsRef<Path>>(mut self, path: &'a P) -> SaveQuery<'a> {
            self.path = Some(path.as_ref());
            self
        }

        pub fn with_friendly_name(mut self, name: &'a str) -> SaveQuery {
            self.friendly_name = Some(name);
            self
        }

        pub fn with_user_id(mut self, id: i32) -> SaveQuery<'a> {
            self.user_id = Some(id);
            self
        }

        pub fn with_uuid(mut self, uuid: &'a str) -> SaveQuery {
            self.uuid = Some(uuid);
            self
        }
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct FileQuery<'a> {
        pub id: Option<i32>,
        pub path: Option<&'a Path>,
        pub hash: Option<&'a [u8]>,
        pub save_id: Option<i32>,
    }

    impl<'a> FileQuery<'a> {
        pub fn new() -> FileQuery<'a> {
            FileQuery {
                id: None,
                path: None,
                hash: None,
                save_id: None,
            }
        }

        pub fn with_id(mut self, id: i32) -> FileQuery<'a> {
            self.id = Some(id);
            self
        }

        pub fn with_path<P: AsRef<Path>>(mut self, path: &'a P) -> FileQuery {
            self.path = Some(path.as_ref());
            self
        }

        pub fn with_hash(mut self, hash: &'a [u8]) -> FileQuery {
            self.hash = Some(hash);
            self
        }

        pub fn with_save_id(mut self, save_id: i32) -> FileQuery<'a> {
            self.save_id = Some(save_id);
            self
        }
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct UserQuery<'a> {
        pub id: Option<i32>,
        pub username: Option<&'a str>,
    }

    impl<'a> UserQuery<'a> {
        pub fn new() -> UserQuery<'a> {
            UserQuery {
                id: None,
                username: None,
            }
        }

        pub fn with_id(mut self, id: i32) -> UserQuery<'a> {
            self.id = Some(id);
            self
        }

        pub fn with_username(mut self, name: &'a str) -> UserQuery {
            self.username = Some(name);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::query::*;
    use super::*;
    use crate::config::Config;
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn convert_u64_to_bytes_valid() {
        let expected: Vec<u8> = vec![162, 237, 204, 196, 230, 7, 254, 234];
        let num: u64 = 16932980336685280674;

        let actual = Archive::u64_to_byte_vec(num).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn calc_hash_from_file() {
        use rand;
        use std::io::Write;

        let test_dir = TempDir::new().unwrap();
        let tmp_path = test_dir.path();

        let file_path: PathBuf = [tmp_path, &PathBuf::from("rand.bin")].iter().collect();
        let bytes: [u8; 32] = rand::random();

        let mut file = File::create(&file_path).unwrap();
        file.write_all(&bytes).unwrap();

        let config = Config::static_config().unwrap();
        let seed = config.xxhash_seed as u64;

        let expected = {
            let mut hasher = XxHash64::with_seed(seed); // Make sure same seed
            hasher.write(&bytes);

            hasher.finish()
        };

        let actual = Archive::calc_hash(&file_path).unwrap();

        test_dir.close().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn compress_and_decompress_directory() {
        use std::io::{Read, Write};

        let test_dir = TempDir::new().unwrap();
        let tmp_path = test_dir.path();

        let archive_name = "archive.tar.zst";
        let src_dir: PathBuf = [tmp_path, &PathBuf::from("test_dir")].iter().collect();
        let archive_path: PathBuf = [tmp_path, &PathBuf::from(archive_name)].iter().collect();
        let copy_dir: PathBuf = [tmp_path, &PathBuf::from("decompress")].iter().collect();

        // Example Directory
        fs::create_dir(&src_dir).unwrap();

        let file1_expected = "This file contains some text";
        let file1_path: PathBuf = [&src_dir, &PathBuf::from("file1.txt")].iter().collect();
        let mut file1 = File::create(file1_path).unwrap();
        file1.write_all(file1_expected.as_bytes()).unwrap();

        let src_sub_dir: PathBuf = [&src_dir, &PathBuf::from("sub_dir")].iter().collect();
        fs::create_dir(&src_sub_dir).unwrap();

        let file2_expected = "This file contains some different text";
        let file2_path: PathBuf = [&src_sub_dir, &PathBuf::from("file2.txt")].iter().collect();
        let mut file2 = File::create(file2_path).unwrap();
        file2.write_all(file2_expected.as_bytes()).unwrap();

        Archive::compress_directory(&src_dir, &archive_path).unwrap();
        Archive::decompress_archive(&archive_path, &copy_dir).unwrap();

        let mut file1_actual = String::new();
        let mut file2_actual = String::new();

        let copy_src_dir = [&copy_dir, &PathBuf::from("test_dir")].iter().collect();
        let file1_copy_path: PathBuf = [&copy_src_dir, &PathBuf::from("file1.txt")]
            .iter()
            .collect();

        let mut file1 = File::open(file1_copy_path).unwrap();
        file1.read_to_string(&mut file1_actual).unwrap();

        let copy_sub_dir: PathBuf = [&copy_src_dir, &PathBuf::from("sub_dir")].iter().collect();
        let file2_copy_path: PathBuf = [&copy_sub_dir, &PathBuf::from("file2.txt")]
            .iter()
            .collect();

        let mut file2 = File::open(file2_copy_path).unwrap();
        file2.read_to_string(&mut file2_actual).unwrap();

        test_dir.close().unwrap();
        assert_eq!(file1_actual, file1_expected);
        assert_eq!(file2_actual, file2_expected);
    }

    #[test]
    fn compress_and_decompress_file() {
        use std::io::{Read, Write};

        let test_dir = TempDir::new().unwrap();
        let tmp_path = test_dir.path();

        let expected: [u8; 32] = rand::random();
        let archive_name = "random.bin.zst";
        let file_path: PathBuf = [tmp_path, &PathBuf::from("random.bin")].iter().collect();
        let actual_path: PathBuf = [tmp_path, &PathBuf::from("actual.bin")].iter().collect();
        let archive_path: PathBuf = [tmp_path, &PathBuf::from(archive_name)].iter().collect();
        let mut file = File::create(&file_path).unwrap();

        file.write_all(&expected).unwrap();

        Archive::compress_file(&file_path, &archive_path).unwrap();
        Archive::decompress_file(&archive_path, &actual_path).unwrap();

        let mut file = File::open(&actual_path).unwrap();

        let mut actual = vec![];
        file.read_to_end(&mut actual).unwrap();

        test_dir.close().unwrap();
        assert_eq!(actual, expected.to_vec());
    }

    #[test]
    fn example_save_query() {
        let path = Path::new("test_location");
        let actual = SaveQuery::new()
            .with_id(1)
            .with_friendly_name("game1")
            .with_uuid("{uuid}")
            .with_path(&path);

        let expected = SaveQuery {
            id: Some(1),
            friendly_name: Some("game1"),
            uuid: Some("{uuid}"),
            path: Some(Path::new("test_location")),
            user_id: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn example_file_query() {
        let hash: [u8; 32] = rand::random();

        let actual = FileQuery::new()
            .with_id(943)
            .with_hash(&hash)
            .with_save_id(2);

        let expected = FileQuery {
            id: Some(943),
            path: None,
            hash: Some(&hash),
            save_id: Some(2),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn example_user_query() {
        let actual = UserQuery::new()
            .with_id(32)
            .with_username("serious_gamer_1");

        let expected = UserQuery {
            id: Some(32),
            username: Some("serious_gamer_1"),
        };

        assert_eq!(actual, expected);
    }
}
