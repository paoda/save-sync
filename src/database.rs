use crate::archive::query::{FileQuery, SaveQuery, UserQuery};
use crate::models::*;
use crate::schema;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;
use std::path::PathBuf;

pub struct Database {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl Database {
    pub fn new(db_url: &PathBuf) -> Database {
        let manager = ConnectionManager::new(db_url.to_str().unwrap());
        let pool = Pool::builder()
            .max_size(15) // TODO: Make Configurable? Is this even necessary?
            .build(manager)
            .unwrap();

        Self::check_db(&pool);

        Database { pool }
    }

    fn check_db(pool: &Pool<ConnectionManager<SqliteConnection>>) {
        let conn = &pool.get().expect("Unable to get DB connection from pool.");

        embed_migrations!("./migrations");
        embedded_migrations::run(conn).expect("Failed to run embedded database migrations.");
    }

    pub fn get_pool(self) -> Pool<ConnectionManager<SqliteConnection>> {
        self.pool
    }

    fn get_conn(&self) -> PooledConnection<ConnectionManager<SqliteConnection>> {
        self.pool
            .get()
            .expect("Unable to get DB connection from pool.")
    }

    fn does_save_exist(&self, path: &str) -> bool {
        use schema::saves::dsl::*;

        let conn = self.get_conn();
        let list: Vec<Save> = saves
            .filter(save_path.eq(path))
            .load(&conn)
            .expect("Unable to query database.");

        !list.is_empty()
    }

    fn does_file_exist(&self, path: &str) -> bool {
        use schema::files::dsl::*;

        let conn = self.get_conn();
        let list: Vec<File> = files
            .filter(file_path.eq(path))
            .load(&conn)
            .expect("Unable to query database.");

        !list.is_empty()
    }

    fn does_user_exist(&self, uname: &str) -> bool {
        use schema::users::dsl::*;

        let conn = self.get_conn();
        let list: Vec<User> = users
            .filter(username.eq(uname))
            .load(&conn)
            .expect("Unable to query database.");

        !list.is_empty()
    }

    pub fn create_save(&self, save: NewSave) {
        // TODO: Return Result
        use schema::saves;

        if !self.does_save_exist(save.save_path) {
            let conn = self.get_conn();

            diesel::insert_into(saves::table)
                .values(&save)
                .execute(&conn)
                .expect("Failed to create save in database.");
        }
    }

    pub fn get_save(&self, query: SaveQuery) -> Option<Save> {
        use schema::saves::dsl::*;

        let err_msg = "Unable to query database.";
        let conn = self.get_conn();
        let mut list: Vec<Save> = vec![];

        if let Some(search_id) = query.id {
            list = saves.filter(id.eq(search_id)).load(&conn).expect(err_msg);
        } else if let Some(name) = query.friendly_name {
            list = saves
                .filter(friendly_name.eq(&name))
                .load(&conn)
                .expect(err_msg);
        } else if let Some(path) = query.path {
            let path_str = path.to_str().unwrap();
            list = saves
                .filter(save_path.eq(path_str))
                .load(&conn)
                .expect(err_msg);
        }

        match list.len() {
            0 => None,
            1 => Some(list.first().unwrap().clone()),
            _ => panic!("Expected 1 save to be found, but found multiple."),
        }
    }

    pub fn get_saves(&self, query: SaveQuery) -> Option<Vec<Save>> {
        use schema::saves::dsl::*;

        let err_msg = "Unable to query database.";
        let conn = self.get_conn();
        let mut list: Vec<Save> = vec![];

        if let Some(search_user_id) = query.user_id {
            list = saves
                .filter(user_id.eq(search_user_id))
                .load(&conn)
                .expect(err_msg);
        }

        match list.is_empty() {
            true => None,
            false => Some(list),
        }
    }

    pub fn get_all_saves(&self) -> Option<Vec<Save>> {
        use schema::saves::dsl::*;

        let conn = self.get_conn();
        let list: Vec<Save> = saves.load(&conn).expect("Unable to query database.");

        match list.is_empty() {
            true => None,
            false => Some(list),
        }
    }

    pub fn update_save(&self, edit: EditSave) {
        // TODO: Return Result
        use schema::saves::dsl::*;

        let conn = self.get_conn();
        let save_id = edit.id;

        diesel::update(saves.filter(id.eq(save_id)))
            .set(&edit)
            .execute(&conn)
            .expect("Failed to update save in database.");
    }

    pub fn delete_save(&self, query: SaveQuery) {
        // TODO: Return Result
        use schema::saves::dsl::*;

        let err_msg = "Unable to delete save from database.";
        let conn = self.get_conn();

        if let Some(search_id) = query.id {
            diesel::delete(saves.filter(id.eq(search_id)))
                .execute(&conn)
                .expect(err_msg);
        } else if let Some(name) = query.friendly_name {
            diesel::delete(saves.filter(friendly_name.eq(&name)))
                .execute(&conn)
                .expect(err_msg);
        } else if let Some(path) = query.path {
            let path_str = path.to_str().unwrap();
            diesel::delete(saves.filter(save_path.eq(path_str)))
                .execute(&conn)
                .expect(err_msg);
        }
    }

    pub fn delete_saves(&self, query: SaveQuery) {
        // TODO: Return result
        use schema::saves::dsl::*;

        let err_msg = "Unable to delete saves from database.";
        let conn = self.get_conn();

        if let Some(search_user_id) = query.user_id {
            diesel::delete(saves.filter(user_id.eq(search_user_id)))
                .execute(&conn)
                .expect(err_msg);
        }
    }

    pub fn create_file(&self, file: NewFile) {
        // TODO: Return result
        use schema::files;

        if !self.does_file_exist(file.file_path) {
            let conn = self.get_conn();

            diesel::insert_into(files::table)
                .values(&file)
                .execute(&conn)
                .expect("Failed to create file in database.");
        }
    }

    pub fn get_file(&self, query: FileQuery) -> Option<File> {
        use schema::files::dsl::*;

        let err_msg = "Unable to query database.";
        let conn = self.get_conn();
        let mut list: Vec<File> = vec![];

        if let Some(search_id) = query.id {
            list = files.filter(id.eq(search_id)).load(&conn).expect(err_msg);
        } else if let Some(path) = query.path {
            let path_str = path.to_str().unwrap();
            list = files
                .filter(file_path.eq(path_str))
                .load(&conn)
                .expect(err_msg);
        } else if let Some(hash) = query.hash {
            list = files
                .filter(file_hash.eq(&hash))
                .load(&conn)
                .expect(err_msg);
        }

        match list.len() {
            0 => None,
            1 => Some(list.first().unwrap().clone()),
            _ => panic!("Expected 1 file to be found, but found multiple."),
        }
    }

    pub fn get_files(&self, query: FileQuery) -> Option<Vec<File>> {
        use schema::files::dsl::*;

        let err_msg = "Unable to query database.";
        let conn = self.get_conn();
        let mut list: Vec<File> = vec![];

        if let Some(search_save_id) = query.save_id {
            list = files
                .filter(save_id.eq(search_save_id))
                .load(&conn)
                .expect(err_msg);
        }

        match list.is_empty() {
            true => None,
            false => Some(list),
        }
    }

    pub fn get_all_files(&self) -> Option<Vec<File>> {
        use schema::files::dsl::*;

        let conn = self.get_conn();
        let list: Vec<File> = files.load(&conn).expect("Unable to query database.");

        match list.is_empty() {
            true => None,
            false => Some(list),
        }
    }

    pub fn update_file(&self, edit: EditFile) {
        // TODO: Return result
        use schema::files::dsl::*;

        let conn = self.get_conn();
        let file_id = edit.id;

        diesel::update(files.filter(id.eq(file_id)))
            .set(&edit)
            .execute(&conn)
            .expect("Failed to update file in database.");
    }

    pub fn delete_file(&self, query: FileQuery) {
        // TODO: Return result
        use schema::files::dsl::*;

        let err_msg = "Unable to delete file from database.";
        let conn = self.get_conn();

        if let Some(search_id) = query.id {
            diesel::delete(files.filter(id.eq(search_id)))
                .execute(&conn)
                .expect(err_msg);
        } else if let Some(path) = query.path {
            let path_str = path.to_str().unwrap();
            diesel::delete(files.filter(file_path.eq(path_str)))
                .execute(&conn)
                .expect(err_msg);
        } else if let Some(hash) = query.hash {
            diesel::delete(files.filter(file_hash.eq(&hash)))
                .execute(&conn)
                .expect(err_msg);
        }
    }

    pub fn delete_files(&self, query: FileQuery) {
        // TODO: Return result
        use schema::files::dsl::*;

        let err_msg = "Unable to delete files from database.";
        let conn = self.get_conn();

        if let Some(search_save_id) = query.save_id {
            diesel::delete(files.filter(save_id.eq(search_save_id)))
                .execute(&conn)
                .expect(err_msg);
        }
    }

    pub fn create_user(&self, user: NewUser) {
        // TODO: Return result
        use schema::users;

        if !self.does_user_exist(user.username) {
            let conn = self.get_conn();

            diesel::insert_into(users::table)
                .values(&user)
                .execute(&conn)
                .expect("Failed to create file in database.");
        }
    }

    pub fn get_user(&self, query: UserQuery) -> Option<User> {
        use schema::users::dsl::*;

        let err_msg = "Unable to query database.";
        let conn = self.get_conn();
        let mut list: Vec<User> = vec![];

        if let Some(search_id) = query.id {
            list = users.filter(id.eq(search_id)).load(&conn).expect(err_msg);
        } else if let Some(uname) = query.username {
            list = users
                .filter(username.eq(&uname))
                .load(&conn)
                .expect(err_msg)
        }

        match list.len() {
            0 => None,
            1 => Some(list.first().unwrap().clone()),
            _ => panic!("Expected 1 user to be found, but found multiple."),
        }
    }

    pub fn get_all_users(&self) -> Option<Vec<User>> {
        use schema::users::dsl::*;

        let conn = self.get_conn();
        let list: Vec<User> = users.load(&conn).expect("Unable to query database.");

        match list.is_empty() {
            true => None,
            false => Some(list),
        }
    }

    pub fn update_user(&self, edit: EditUser) {
        // TODO: Return result
        use schema::users::dsl::*;

        let conn = self.get_conn();
        let user_id = edit.id;

        diesel::update(users.filter(id.eq(user_id)))
            .set(&edit)
            .execute(&conn)
            .expect("Failed to update user in database.");
    }

    pub fn delete_user(&self, query: UserQuery) {
        use schema::users::dsl::*;

        let err_msg = "Unable to delete user from database.";
        let conn = self.get_conn();

        if let Some(search_id) = query.id {
            diesel::delete(users.filter(id.eq(search_id)))
                .execute(&conn)
                .expect(err_msg);
        } else if let Some(uname) = query.username {
            diesel::delete(users.filter(username.eq(&uname)))
                .execute(&conn)
                .expect(err_msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // imports archive queries, model structs diesel prelude etc.
    use chrono::Utc;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn write_to_and_migrate_database() {
        let test_folder_id = "new_db";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let _db = Database::new(&db_path);

        let result = db_path.exists();

        destroy_test_dir(test_folder_id);
        assert!(result);
    }

    #[test]
    fn does_save_exist_true() {
        let test_folder_id = "save_does_exist";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let time = Utc::now().naive_utc();

        let save = NewSave {
            friendly_name: "test_game",
            save_path: "/home/user/Documents/test_game",
            backup_path: "/home/user/.local/share/save-sync/{uuid}/test_game",
            uuid: "{uuid}",
            user_id: 1,
            created_at: time,
            modified_at: time,
        };

        db.create_save(save);
        let result = db.does_save_exist(save.save_path);

        destroy_test_dir(test_folder_id);
        assert!(result);
    }

    #[test]
    fn does_save_exist_false() {
        let test_folder_id = "save_does_not_exist";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let path = "/home/user/Documents/test_game";
        let result = db.does_file_exist(path);

        destroy_test_dir(test_folder_id);
        assert!(!result);
    }

    #[test]
    fn does_file_exist_true() {
        let test_folder_id = "file_does_exist";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let time = Utc::now().naive_utc();

        let file = NewFile {
            file_path: "/home/user/Documents/test_game/00.sav",
            file_hash: &vec![1, 2, 3, 4, 5],
            save_id: 1,
            created_at: time,
            modified_at: time,
        };

        db.create_file(file);
        let result = db.does_file_exist(file.file_path);

        destroy_test_dir(test_folder_id);
        assert!(result);
    }

    #[test]
    fn does_file_exist_false() {
        let test_folder_id = "file_does_not_exist";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let path = "/home/user/Documents/test_game/00.sav";
        let result = db.does_file_exist(path);

        destroy_test_dir(test_folder_id);
        assert!(!result);
    }

    #[test]
    fn does_user_exist_true() {
        let test_folder_id = "uesr_does_exist";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let time = Utc::now().naive_utc();

        let user = NewUser {
            username: "DarkFlameMaster",
            created_at: time,
            modified_at: time,
        };

        db.create_user(user);
        let result = db.does_user_exist(user.username);

        destroy_test_dir(test_folder_id);
        assert!(result);
    }

    #[test]
    fn does_user_exist_false() {
        let test_folder_id = "user_does_not_exist";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let username = "DarkFlameMaster";
        let result = db.does_user_exist(username);

        destroy_test_dir(test_folder_id);
        assert!(!result);
    }

    #[test]
    fn create_new_save() {
        let test_folder_id = "new_save";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let time = Utc::now().naive_utc();

        let expected = NewSave {
            friendly_name: "test_game",
            save_path: "/home/user/Documents/test_game",
            backup_path: "/home/user/.local/share/save-sync/{uuid}/test_game",
            uuid: "{uuid}",
            user_id: 1,
            created_at: time,
            modified_at: time,
        };

        db.create_save(expected);

        let conn = db.get_conn();

        let path = expected.save_path;
        let list: Vec<Save> = {
            use crate::schema::saves::dsl::*;
            saves.filter(save_path.eq(path)).load(&conn).unwrap()
        };

        destroy_test_dir(test_folder_id);
        assert!(list.len() == 1);
        let actual = list.first().unwrap().clone();

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_save_success() {
        // FIXME: With get_save_success, get_file_success and get_user_success
        // we only test one out of many different queries we could come up with
        // We might want to consider writing tests for all of those conditions,
        // no matter how tedious it may be

        use crate::schema::saves;

        let test_folder_id = "get_save_success";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let time = Utc::now().naive_utc();

        let expected = NewSave {
            friendly_name: "test_game",
            save_path: "/home/user/Documents/test_game",
            backup_path: "/home/user/.local/share/save-sync/{uuid}/test_game",
            uuid: "{uuid}",
            user_id: 1,
            created_at: time,
            modified_at: time,
        };

        let conn = db.get_conn();

        diesel::insert_into(saves::table)
            .values(&expected)
            .execute(&conn)
            .unwrap();

        let query = SaveQuery::new().with_friendly_name("test_game");

        let actual = db.get_save(query).unwrap();

        destroy_test_dir(test_folder_id);

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_save_failure() {
        let test_folder_id = "get_save_failure";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let query = SaveQuery::new().with_friendly_name("not_in_db");

        destroy_test_dir(test_folder_id);
        assert!(db.get_save(query).is_none())
    }

    #[test]
    fn get_saves_success() {
        use crate::schema::saves;

        let test_folder_id = "get_saves_success";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let time = Utc::now().naive_utc();

        let expected1 = NewSave {
            friendly_name: "test_game",
            save_path: "/home/user/Documents/test_game",
            backup_path: "/home/user/.local/share/save-sync/{uuid}/test_game",
            uuid: "{uuid}",
            user_id: 1,
            created_at: time,
            modified_at: time,
        };

        let time = Utc::now().naive_utc();

        let expected2 = NewSave {
            friendly_name: "other_game",
            save_path: "/home/user/Documents/other_game",
            backup_path: "/home/user/.local/share/save-sync/{other_uuid}/other_game",
            uuid: "{other_uuid}",
            user_id: 1,
            created_at: time,
            modified_at: time,
        };

        let conn = db.get_conn();

        // Batch Inserts are not supported in diesel (when it comes to SQlite)
        diesel::insert_into(saves::table)
            .values(&expected1)
            .execute(&conn)
            .unwrap();

        diesel::insert_into(saves::table)
            .values(&expected2)
            .execute(&conn)
            .unwrap();

        let query = SaveQuery::new().with_user_id(1);
        let saves = db.get_saves(query).unwrap();

        destroy_test_dir(test_folder_id);

        assert!(saves.len() == 2);
        let actual1: Save = saves.get(0).unwrap().clone();
        let actual2: Save = saves.get(1).unwrap().clone();

        assert_eq!(actual1, expected1);
        assert_eq!(actual2, expected2);
    }

    #[test]
    fn get_saves_failure() {
        let test_folder_id = "get_saves_failure";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let query = SaveQuery::new().with_user_id(1);
        let saves = db.get_saves(query);

        destroy_test_dir(test_folder_id);
        assert!(saves.is_none());
    }

    #[test]
    #[ignore]
    fn get_all_saves_success() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn get_all_saves_failure() {
        unimplemented!();
    }

    #[test]
    #[ignore]
    fn update_save_succss() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn update_save_failure() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_save_success() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_save_failure() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_saves_success() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_saves_failure() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn create_new_file() {
        unimplemented!()
    }

    #[test]
    fn get_file_success() {
        use crate::schema::files;

        let test_folder_id = "get_file_success";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let time = Utc::now().naive_utc();

        let expected = NewFile {
            file_path: "/home/user/Documents/test_game/00.sav",
            file_hash: &vec![1, 2, 3, 4, 5],
            save_id: 1,
            created_at: time,
            modified_at: time,
        };

        let conn = db.get_conn();

        diesel::insert_into(files::table)
            .values(&expected)
            .execute(&conn)
            .unwrap();

        let path = PathBuf::from("/home/user/Documents/test_game/00.sav");
        let query = FileQuery::new().with_path(path);

        let actual = db.get_file(query).unwrap();

        destroy_test_dir(test_folder_id);

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_file_failure() {
        let test_folder_id = "get_file_failure";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let query = FileQuery::new().with_hash(vec![1, 1, 0, 3, 7]);

        destroy_test_dir(test_folder_id);
        assert!(db.get_file(query).is_none())
    }

    #[test]
    fn get_files_success() {
        use crate::schema::files;

        let test_folder_id = "get_files_success";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let time = Utc::now().naive_utc();

        let expected1 = NewFile {
            file_path: "/home/user/Documents/test_game/00.sav",
            file_hash: &vec![1, 2, 3, 4, 5],
            save_id: 1,
            created_at: time,
            modified_at: time,
        };

        let time = Utc::now().naive_utc();

        let expected2 = NewFile {
            file_path: "/home/user/Documents/test_game/01.sav",
            file_hash: &vec![5, 4, 3, 2, 1],
            save_id: 1,
            created_at: time,
            modified_at: time,
        };

        let conn = db.get_conn();

        // Batch Inserts are not supported in diesel (when it comes to SQlite)
        diesel::insert_into(files::table)
            .values(&expected1)
            .execute(&conn)
            .unwrap();

        diesel::insert_into(files::table)
            .values(&expected2)
            .execute(&conn)
            .unwrap();

        let query = FileQuery::new().with_save_id(1);
        let files = db.get_files(query).unwrap();

        destroy_test_dir(test_folder_id);

        assert!(files.len() == 2);
        let actual1 = files.get(0).unwrap().clone();
        let actual2 = files.get(1).unwrap().clone();

        assert_eq!(actual1, expected1);
        assert_eq!(actual2, expected2);
    }

    #[test]
    fn get_files_failure() {
        let test_folder_id = "get_files_failure";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let query = FileQuery::new().with_save_id(1);
        let files = db.get_files(query);

        destroy_test_dir(test_folder_id);
        assert!(files.is_none());
    }

    #[test]
    #[ignore]
    fn get_all_files_success() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn get_all_files_failure() {
        unimplemented!();
    }

    #[test]
    #[ignore]
    fn update_file_succss() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn update_file_failure() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_file_success() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_file_failure() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_files_success() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_files_failure() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn create_new_user() {
        unimplemented!()
    }

    #[test]
    fn get_user_success() {
        use crate::schema::users;

        let test_folder_id = "get_user_success";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let time = Utc::now().naive_utc();

        let expected = NewUser {
            username: "DarkFlameMaster",
            created_at: time,
            modified_at: time,
        };

        let conn = db.get_conn();

        diesel::insert_into(users::table)
            .values(&expected)
            .execute(&conn)
            .unwrap();

        let query = UserQuery::new().with_username("DarkFlameMaster");

        let actual = db.get_user(query).unwrap();

        destroy_test_dir(test_folder_id);

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_user_failure() {
        let test_folder_id = "get_user_failure";
        let tmp_dir = setup_test_dir(test_folder_id);

        let db_path: PathBuf = [&tmp_dir, &PathBuf::from("test.db")].iter().collect();
        let db = Database::new(&db_path);

        let query = UserQuery::new().with_username("nonexistent_username");

        destroy_test_dir(test_folder_id);
        assert!(db.get_user(query).is_none())
    }

    #[test]
    #[ignore]
    fn get_all_users_success() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn get_all_users_failure() {
        unimplemented!();
    }

    #[test]
    #[ignore]
    fn update_user_succss() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn update_user_failure() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_user_success() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_user_failure() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_users_success() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn delete_users_failure() {
        unimplemented!()
    }

    fn setup_test_dir(id: &str) -> PathBuf {
        let test_dir = PathBuf::from(format!("./tmp_dir_database_{}", id));

        if test_dir.exists() {
            destroy_test_dir(id);
        }

        fs::create_dir(&test_dir).unwrap();
        test_dir
    }

    fn destroy_test_dir(id: &str) {
        let test_dir = PathBuf::from(format!("./tmp_dir_database_{}", id));

        fs::remove_dir_all(test_dir).unwrap();
    }
}
