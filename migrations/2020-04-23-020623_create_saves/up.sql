-- Your SQL goes here
CREATE TABLE saves (
    id INTEGER NOT NULL PRIMARY KEY,
    friendly_name TEXT NOT NULL,
    save_path TEXT NOT NULL,
    backup_path TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    modified_at DATETIME NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);
