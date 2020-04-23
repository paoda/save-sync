-- Your SQL goes here
CREATE TABLE files (
    id INTEGER NOT NULL PRIMARY KEY,
    file_path TEXT NOT NULL,
    file_hash BLOB NOT NULL,
    uuid CHAR(36) NOT NULL,
    save_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    modified_at DATETIME NOT NULL,
    FOREIGN KEY(save_id) REFERENCES saves(id)
);
