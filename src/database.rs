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

        Database { pool }
    }

    pub fn get_pool(self) -> Pool<ConnectionManager<SqliteConnection>> {
        self.pool
    }

    fn get_conn(&self) -> PooledConnection<ConnectionManager<SqliteConnection>> {
        self.pool
            .get()
            .expect("Unable to get DB connection from pool.")
    }

    fn does_save_exist(&self, save: &NewSave) -> bool {
        use schema::saves::dsl::*;

        let path = save.save_path;
        let conn = self.get_conn();
        let list: Vec<Save> = saves
            .filter(save_path.eq(path))
            .load(&conn)
            .expect("Unable to query database.");

        !list.is_empty()
    }

    fn does_file_exist(&self, file: &NewFile) -> bool {
        use schema::files::dsl::*;

        let path = file.file_path;
        let conn = self.get_conn();
        let list: Vec<File> = files
            .filter(file_path.eq(path))
            .load(&conn)
            .expect("Unable to query database.");

        !list.is_empty()
    }

    fn does_user_exist(&self, user: &NewUser) -> bool {
        use schema::users::dsl::*;

        let uname = user.username;
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

        if !self.does_save_exist(&save) {
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

        if !self.does_file_exist(&file) {
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

        if !self.does_user_exist(&user) {
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
    use std::path::PathBuf;

    fn setup_test_dir(id: &str) -> PathBuf {
        use std::fs::create_dir;
        let test_dir = PathBuf::from(format!("./tmp_dir_database_{}", id));

        if test_dir.exists() {
            destroy_test_dir(id);
        }

        create_dir(&test_dir).unwrap();
        test_dir
    }

    fn destroy_test_dir(id: &str) {
        use std::fs::remove_dir_all;
        let test_dir = PathBuf::from(format!("./tmp_dir_database_{}", id));

        remove_dir_all(test_dir).unwrap();
    }
}
