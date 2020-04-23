-- Your SQL goes here
CREATE TABLE users (
  id INTEGER NOT NULL PRIMARY KEY,
  username VARCHAR(30) NOT NULL,
  created_at DATETIME NOT NULL,
  modified_at DATETIME NOT NULL
);