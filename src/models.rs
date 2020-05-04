use crate::schema::{files, saves, users};
use chrono::naive::NaiveDateTime;

#[derive(Clone, Debug, Queryable, Insertable)]
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

#[derive(Clone, Copy, Debug, AsChangeset)]
#[table_name = "saves"]
pub struct EditSave<'a> {
    pub id: i32,
    pub friendly_name: Option<&'a str>,
    pub save_path: Option<&'a str>,
    pub modified_at: NaiveDateTime,
}

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

#[derive(Clone, Debug, Queryable, Insertable)]
pub struct File {
    pub id: i32,
    pub file_path: String,
    pub file_hash: Vec<u8>,
    pub save_id: i32,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

#[derive(Clone, Copy, Debug, Insertable)]
#[table_name = "files"]
pub struct NewFile<'a> {
    pub file_path: &'a str,
    pub file_hash: &'a [u8],
    pub save_id: i32,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

#[derive(Clone, Copy, Debug, AsChangeset)]
#[table_name = "files"]
pub struct EditFile<'a> {
    pub id: i32,
    pub file_hash: &'a [u8],
    pub modified_at: NaiveDateTime,
}

impl PartialEq<NewFile<'_>> for File {
    fn eq(&self, other: &NewFile) -> bool {
        self.file_path == other.file_path
            && self.file_hash == other.file_hash
            && self.save_id == other.save_id
            && self.created_at == other.created_at
            && self.modified_at == other.modified_at
    }
}

#[derive(Clone, Debug, Queryable, Insertable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

#[derive(Clone, Copy, Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

#[derive(Clone, Copy, Debug, AsChangeset)]
#[table_name = "users"]
pub struct EditUser<'a> {
    pub id: i32,
    pub username: Option<&'a str>,
    pub modified_at: NaiveDateTime,
}

impl PartialEq<NewUser<'_>> for User {
    fn eq(&self, other: &NewUser) -> bool {
        self.username == other.username
            && self.created_at == other.created_at
            && self.modified_at == other.modified_at
    }
}
