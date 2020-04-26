#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel_migrations;

pub mod archive;
pub mod config;
pub mod database;
mod models;
mod schema;
