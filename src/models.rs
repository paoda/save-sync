use crate::schema::{files, saves, users};
use chrono::naive::NaiveDateTime;

/// Represents a Save in the Database
///
/// # Properties
/// * `id` - The ID of the Save in the Database
/// * `friendly_name` - A Convenient name of the save which will be useful when manually querying the database as a user
/// * `save_path` - A UTF-8 String which represents the root of the **original** save files
/// * `backup_path` - A UTF-8 String which represents the root of the **local** backup of save files
/// * `uuid` - The UUID associated with this Save
/// * `created_at` - A timestamp which represents when this save was created in the database
/// * `modified_at` - A timestamp which represents when this save was last edited in the database
#[derive(Clone, Debug, Eq, PartialEq, Queryable, Insertable)]
pub struct Save {
    pub id: i32,
    pub friendly_name: String,
    pub save_path: String,
    pub backup_path: String,
    pub uuid: String,
    pub user_id: i32,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

// Allows for a comparison between a Path and a Save using the `==` operator
impl PartialEq<std::path::Path> for Save {
    fn eq(&self, other: &std::path::Path) -> bool {
        if let Some(path) = other.to_str() {
            self.save_path == path
        } else {
            false
        }
    }
}

/// Represents a (to-be) newly Created Save
/// Note: With the exception of `created_at` and `modified_at` every property in this struct contains borrowed data.
/// # Properties
/// * `friendly_name` - A Convenient name of the save which will be useful when manually querying the database as a user
/// * `save_path` - A UTF-8 String which represents the root of the **original** save files
/// * `backup_path` - A UTF-8 String which represents the root of the **local** backup of save files
/// * `uuid` - The UUID associated with this Save
/// * `created_at` - A timestamp which represents when this save was created in the database
/// * `modified_at` - A timestamp which represents when this save was last edited in the database
#[derive(Clone, Copy, Debug, Insertable)]
#[table_name = "saves"]
pub struct NewSave<'a> {
    pub friendly_name: &'a str,
    pub save_path: &'a str,
    pub backup_path: &'a str,
    pub uuid: &'a str,
    pub user_id: i32,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

/// Represents a Changelist of a Save
/// Note: With the excpetion of `modified_at`, every property in this struct contains borrowed data.
/// # Properties
/// * `id` - The ID of the Save in the Database
/// * `friendly_name` - A Convenient name of the save which will be useful when manually querying the database as a user
/// * `save_path` - A UTF-8 String which represents the root of the **original** save files
/// * `modified_at` - A timestamp which represents when this save was last edited in the database
#[derive(Clone, Copy, Debug, AsChangeset)]
#[table_name = "saves"]
pub struct EditSave<'a> {
    pub id: i32,
    pub friendly_name: Option<&'a str>,
    pub save_path: Option<&'a str>,
    pub modified_at: NaiveDateTime,
}

// Allows for a comparison between a NewSave and an existing Save using the `==` operator
impl PartialEq<NewSave<'_>> for Save {
    fn eq(&self, other: &NewSave) -> bool {
        self.friendly_name == other.friendly_name
            && self.save_path == other.save_path
            && self.backup_path == other.backup_path
            && self.uuid == other.uuid
            && self.user_id == other.user_id
            && self.created_at == other.created_at
            && self.modified_at == other.modified_at
    }
}

/// Represents a File in the Databse
/// # Properties
/// * `id` - The ID of the File in the Database
/// * `file_path` - A UTF-8 String that represents the **original** location of the file
/// * `file_hash` - A u64 (calculated using xx_hash) which has been turned into a little endian byte array
/// * `save_id` - The ID of which this File belongs to
/// * `created_at` - A timestamp that represents when this File was created in the database
/// * `modified_at` - A timestamp that represents when this File as last modified in the database.
#[derive(Clone, Debug, Eq, PartialEq, Queryable, Insertable)]
pub struct File {
    pub id: i32,
    pub file_path: String,
    pub file_hash: Vec<u8>,
    pub save_id: i32,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

// Allows for a comparison between a Path and a File using the `==` operator
impl PartialEq<std::path::Path> for File {
    fn eq(&self, other: &std::path::Path) -> bool {
        if let Some(path) = other.to_str() {
            self.file_path == path
        } else {
            false
        }
    }
}

/// Represents a (to-be) newly created File
/// Note: With the exception of `created_at` and `modified_at`, all properties in this struct contain borrowed data.
/// # Properties
/// * `file_path` - A UTF-8 String that represents the **original** location of the file
/// * `file_hash` - A u64 (calculated using xx_hash) which has been turned into a little endian byte array
/// * `save_id` - The ID of which this File belongs to
/// * `created_at` - A timestamp that represents when this File was created in the database
/// * `modified_at` - A timestamp that represents when this File as last modified in the database.
#[derive(Clone, Copy, Debug, Insertable)]
#[table_name = "files"]
pub struct NewFile<'a> {
    pub file_path: &'a str,
    pub file_hash: &'a [u8],
    pub save_id: i32,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

/// Represents a ChangeList of a File
/// # Note: With the exception of `modified_at`
/// * `id` - The ID of the File in the Database
/// * `file_hash` - A u64 (calculated using xx_hash) which has been turned into a little endian byte array
/// * `modified_at` - A timestamp that represents when this File was last modified in the database.
#[derive(Clone, Copy, Debug, AsChangeset)]
#[table_name = "files"]
pub struct EditFile<'a> {
    pub id: i32,
    pub file_hash: &'a [u8],
    pub modified_at: NaiveDateTime,
}

// Allows for a comparison between a NewFile and an existing file using the `==` operator
impl PartialEq<NewFile<'_>> for File {
    fn eq(&self, other: &NewFile) -> bool {
        self.file_path == other.file_path
            && self.file_hash == other.file_hash
            && self.save_id == other.save_id
            && self.created_at == other.created_at
            && self.modified_at == other.modified_at
    }
}

/// Represents a User
/// # Properties
/// * `id` - The ID of the User in the database
/// * `username` - The Username of the User
/// * `created_at` - A timestamp that repesents when this User was created in the database
/// * `modified_at` - A timestamp that represents when this User was last modified in the database
#[derive(Clone, Debug, Eq, PartialEq, Queryable, Insertable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

/// Represents a (to-be) newly created User
/// Note: username is a property that contains borrowed data
/// # Properties
/// * `username` - The Username of the User
/// * `created_at` - A timestamp that repesents when this User was created in the database
/// * `modified_at` - A timestamp that represents when this User was last modified in the database
#[derive(Clone, Copy, Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

/// Represents a ChangeList of a User
/// Note: username is a property that contains borrowed data
/// # Properties
/// * `id` - The ID of the User in the database
/// * `username` - The Username of the user
/// * `modified_at` - A timestamp that represents when this User was lsat modified inthe database
#[derive(Clone, Copy, Debug, AsChangeset)]
#[table_name = "users"]
pub struct EditUser<'a> {
    pub id: i32,
    pub username: Option<&'a str>,
    pub modified_at: NaiveDateTime,
}

// Allows for a comparison between a NewUser and an existing User using the `==` operator
impl PartialEq<NewUser<'_>> for User {
    fn eq(&self, other: &NewUser) -> bool {
        self.username == other.username
            && self.created_at == other.created_at
            && self.modified_at == other.modified_at
    }
}
