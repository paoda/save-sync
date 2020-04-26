use crate::schema::{files, saves, users};
use chrono::naive::NaiveDateTime;

#[derive(Clone, Queryable, Insertable)]
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

#[derive(Insertable)]
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

#[derive(AsChangeset)]
#[table_name = "saves"]
pub struct EditSave<'a> {
    pub id: i32,
    pub friendly_name: Option<&'a str>,
    pub save_path: Option<&'a str>,
    pub modified_at: NaiveDateTime,
}

#[derive(Clone, Queryable, Insertable)]
pub struct File {
    pub id: i32,
    pub file_path: String,
    pub file_hash: Vec<u8>,
    pub save_id: i32,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "files"]
pub struct NewFile<'a> {
    pub file_path: &'a str,
    pub file_hash: &'a [u8],
    pub save_id: i32,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

#[derive(AsChangeset)]
#[table_name = "files"]
pub struct EditFile<'a> {
    pub id: i32,
    pub file_hash: &'a [u8],
    pub modified_at: NaiveDateTime,
}

#[derive(Clone, Queryable, Insertable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub created_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
}

#[derive(AsChangeset)]
#[table_name = "users"]
pub struct EditUser<'a> {
    pub id: i32,
    pub username: Option<&'a str>,
    pub modified_at: NaiveDateTime,
}
