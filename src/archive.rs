use chrono::prelude::{NaiveDateTime, Utc};
use std::fs::File;
use std::hash::Hasher;
use std::path::PathBuf;
use tar::Archive as TarArchive;
use tar::Builder as TarBuilder;
use twox_hash::XxHash64;

pub struct Archive {}

impl Archive {
    pub fn new() -> Archive {
        Archive {}
    }

    pub fn u64_to_byte_vec(num: u64) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};

        let mut bytes: Vec<u8> = vec![];

        match bytes.write_u64::<LittleEndian>(num) {
            Ok(_) => bytes,
            Err(_err) => panic!("Unable to convert u64 into Vec<u8>"),
        }
    }

    pub fn calc_hash(path: &PathBuf) -> u64 {
        use std::io::Read;

        let seed = 1337;

        let mut hasher = XxHash64::with_seed(seed);
        let chunk_size = 0x4000;
        let file_result = File::open(path);
        let panic_msg = format!("Failed to Stream data from \"{}\"", &path.to_string_lossy());

        match file_result {
            Ok(mut file) => {
                loop {
                    let mut chunk = Vec::with_capacity(chunk_size);
                    let n = file
                        .by_ref()
                        .take(chunk_size as u64)
                        .read_to_end(&mut chunk)
                        .expect(&panic_msg);
                    if n == 0 {
                        break;
                    }
                    hasher.write(&chunk);
                }
                hasher.finish()
            }
            Err(err) => panic!("{}", err),
        }
    }

    pub fn compress_directory(source: &PathBuf, target: &PathBuf) {
        let tar_file = File::create(target).unwrap();
        let zstd_encoder = zstd::stream::Encoder::new(tar_file, 0).unwrap();
        let mut archive = TarBuilder::new(zstd_encoder);

        let base_name = source.file_name().unwrap().to_str(); // TODO: Handle Unwrap

        match base_name {
            Some(name) => {
                archive.append_dir_all(name, source).unwrap();
                let zstd_encoder = archive.into_inner().unwrap();

                zstd_encoder.finish().unwrap();
            }
            None => {
                panic!(
                    "Failed to Convert the File name of {} into a Valid UTF-8 String",
                    source.to_string_lossy()
                );
            }
        }
    }

    pub fn compress_file(source: &PathBuf, target: &PathBuf) {
        use std::io::{Read, Write};

        let buf_size = 0x4000;
        let mut file = File::open(source).unwrap();
        let compressed_file = File::create(target).unwrap();
        let mut zstd_encoder = zstd::stream::Encoder::new(compressed_file, 0).unwrap();

        loop {
            let mut chunk = Vec::with_capacity(buf_size);
            let len = std::io::Read::by_ref(&mut file)
                .take(buf_size as u64)
                .read_to_end(&mut chunk)
                .unwrap();
            if len == 0 {
                break;
            }

            zstd_encoder.write_all(&chunk).unwrap();
        }

        zstd_encoder.finish().unwrap();
    }

    pub fn decompress_archive(source: &PathBuf, target: &PathBuf) {
        let source_file = File::open(source).unwrap();
        let zstd_decoder = zstd::stream::Decoder::new(source_file).unwrap();
        let mut archive = TarArchive::new(zstd_decoder);

        archive.unpack(target).unwrap();
    }

    pub fn decompress_file(source: &PathBuf, target: &PathBuf) {
        use std::io::{Read, Write};

        let buf_size = 0x4000;
        let file = File::open(source).unwrap();
        let zstd_decoder = zstd::stream::Decoder::new(file).unwrap();

        let mut decoded_buf = zstd_decoder.finish();
        let mut target_file = File::create(target).unwrap();

        loop {
            let mut chunk = Vec::with_capacity(buf_size);
            let len = std::io::Read::by_ref(&mut decoded_buf)
                .take(buf_size as u64)
                .read_to_end(&mut chunk)
                .unwrap();
            if len == 0 {
                break;
            }

            target_file.write_all(&chunk).unwrap();
        }
    }

    /// Gets a unix time stamp in UTC±0:00
    pub fn get_utc_unix_time() -> NaiveDateTime {
        Utc::now().naive_utc()
    }
}

pub mod query {
    use std::path::PathBuf;

    #[derive(Debug, PartialEq, Eq)]
    pub struct SaveQuery {
        pub id: Option<i32>,
        pub friendly_name: Option<String>,
        pub path: Option<PathBuf>,
        pub user_id: Option<i32>,
    }

    impl SaveQuery {
        pub fn new() -> SaveQuery {
            SaveQuery {
                id: None,
                friendly_name: None,
                path: None,
                user_id: None,
            }
        }
        pub fn with_id(mut self, id: i32) -> SaveQuery {
            self.id = Some(id);
            self
        }

        pub fn with_path(mut self, path: PathBuf) -> SaveQuery {
            self.path = Some(path);
            self
        }

        pub fn with_friendly_name(mut self, name: &str) -> SaveQuery {
            self.friendly_name = Some(name.to_string()); // FIXME: Memory?s
            self
        }

        pub fn with_user_id(mut self, id: i32) -> SaveQuery {
            self.user_id = Some(id);
            self
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct FileQuery {
        pub id: Option<i32>,
        pub path: Option<PathBuf>,
        pub hash: Option<Vec<u8>>,
        pub save_id: Option<i32>,
    }

    impl FileQuery {
        pub fn new() -> FileQuery {
            FileQuery {
                id: None,
                path: None,
                hash: None,
                save_id: None,
            }
        }

        pub fn with_id(mut self, id: i32) -> FileQuery {
            self.id = Some(id);
            self
        }

        pub fn with_path(mut self, path: PathBuf) -> FileQuery {
            self.path = Some(path);
            self
        }

        pub fn with_hash(mut self, hash: Vec<u8>) -> FileQuery {
            self.hash = Some(hash);
            self
        }

        pub fn with_save_id(mut self, save_id: i32) -> FileQuery {
            self.save_id = Some(save_id);
            self
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct UserQuery {
        pub id: Option<i32>,
        pub username: Option<String>,
    }

    impl UserQuery {
        pub fn new() -> UserQuery {
            UserQuery {
                id: None,
                username: None,
            }
        }

        pub fn with_id(mut self, id: i32) -> UserQuery {
            self.id = Some(id);
            self
        }

        pub fn with_username(mut self, name: &str) -> UserQuery {
            self.username = Some(name.to_string()); // FIXME: Memory?
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::query::*;
    use super::*;
    use std::fs::File;

    #[test]
    fn convert_u64_to_bytes_valid() {
        let expected: Vec<u8> = vec![162, 237, 204, 196, 230, 7, 254, 234];
        let num: u64 = 16932980336685280674;

        let actual = Archive::u64_to_byte_vec(num);

        assert_eq!(actual, expected);
    }

    #[test]
    fn calc_hash_from_file() {
        use rand;
        use std::io::Write;

        let test_folder_id = "hash";
        let tmp_path = setup_test_dir(test_folder_id);

        let file_path: PathBuf = [&tmp_path, &PathBuf::from("rand.bin")].iter().collect();
        let bytes: [u8; 32] = rand::random();

        let mut file = File::create(&file_path).unwrap();
        file.write_all(&bytes).unwrap();

        let expected = {
            let mut hasher = XxHash64::with_seed(1337); // Make sure same seed
            hasher.write(&bytes);

            hasher.finish()
        };

        let actual = Archive::calc_hash(&file_path);

        destroy_test_dir(test_folder_id);
        assert_eq!(actual, expected);
    }

    #[test]
    fn compress_and_decompress_directory() {
        unimplemented!();
    }

    #[test]
    fn compress_and_decompress_file() {
        use std::io::{Read, Write};

        let test_folder_id = "compress";
        let tmp_path = setup_test_dir(test_folder_id);

        let expected: [u8; 32] = rand::random();
        let archive_name = "archive.tar.zst";
        let file_path: PathBuf = [&tmp_path, &PathBuf::from("rand.bin")].iter().collect();
        let actual_path: PathBuf = [&tmp_path, &PathBuf::from("actual.bin")].iter().collect();
        let archive_path: PathBuf = [&tmp_path, &PathBuf::from(archive_name)].iter().collect();
        let mut file = File::create(&file_path).unwrap();

        file.write_all(&expected).unwrap();

        Archive::compress_file(&file_path, &archive_path);
        Archive::decompress_file(&archive_path, &actual_path);

        let mut file = File::open(&actual_path).unwrap();

        let mut actual = vec![];
        file.read_to_end(&mut actual).unwrap();

        destroy_test_dir(test_folder_id);
        assert_eq!(actual, expected.to_vec());
    }

    #[test]
    fn example_save_query() {
        let actual = SaveQuery::new()
            .with_id(1)
            .with_friendly_name("game1")
            .with_path(PathBuf::from("test_location"));

        let expected = SaveQuery {
            id: Some(1),
            friendly_name: Some(String::from("game1")),
            path: Some(PathBuf::from("test_location")),
            user_id: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn example_file_query() {
        let actual = FileQuery::new()
            .with_id(943)
            .with_hash(vec![1, 2, 3, 4, 5])
            .with_save_id(2);

        let expected = FileQuery {
            id: Some(943),
            path: None,
            hash: Some(vec![1, 2, 3, 4, 5]),
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
            username: Some(String::from("serious_gamer_1")),
        };

        assert_eq!(actual, expected);
    }

    fn setup_test_dir(id: &str) -> PathBuf {
        use std::fs::create_dir;
        let test_dir = PathBuf::from(format!("./tmp_dir_archive_{}", id));

        if test_dir.exists() {
            destroy_test_dir(id);
        }

        create_dir(&test_dir).unwrap();
        test_dir
    }

    fn destroy_test_dir(id: &str) {
        use std::fs::remove_dir_all;
        let test_dir = PathBuf::from(format!("./tmp_dir_archive_{}", id));

        remove_dir_all(test_dir).unwrap();
    }
}
