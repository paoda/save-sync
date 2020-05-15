use clap::{App, Arg, ArgMatches, SubCommand};
use cli::archive::Archive;
use save_sync::config::Config;
use save_sync::models::{NewUser, User};
use save_sync::ConfigManager;
use save_sync::Database;
use std::path::PathBuf;

fn main() {
    let matches = App::new("Save Sync")
        .version("0.1.0")
        .author("paoda <musukarekai@gmail.com>")
        .about("Manages saved game data across platforms.")
        .subcommand(
            SubCommand::with_name("info")
                .about("Display information about saved data.")
                .arg(
                    Arg::with_name("friendly")
                        .short("f")
                        .long("friendly")
                        .value_name("NAME")
                        .takes_value(true)
                        .help("The friendly name of the saved data."),
                )
                .arg(
                    Arg::with_name("path")
                        .help("the path of the saved data.")
                        .index(1)
                        .required_unless("friendly"),
                )
                .arg(
                    Arg::with_name("delta")
                        .short("d")
                        .long("delta")
                        .help("Determines which files have changed since last backup."),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Removes path from list of watched paths.")
                .alias("del")
                .arg(
                    Arg::with_name("friendly")
                        .short("f")
                        .long("friendly")
                        .value_name("NAME")
                        .takes_value(true)
                        .help("The friendly name of the saved data."),
                )
                .arg(
                    Arg::with_name("path")
                        .help("the path which will be deleted")
                        .index(1)
                        .required_unless("friendly"),
                ),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds path to list of watched paths.")
                .arg(
                    Arg::with_name("friendly")
                        .short("f")
                        .long("friendly")
                        .value_name("NAME")
                        .takes_value(true)
                        .help("The friendly name of the saved data"),
                )
                .arg(
                    Arg::with_name("path")
                        .help("The path which will be added")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("list").about("Lists every tracked save directory / file"),
        )
        .subcommand(
            SubCommand::with_name("update")
                .about("Updates Backup of save.")
                .arg(
                    Arg::with_name("friendly")
                        .short("f")
                        .long("friendly")
                        .value_name("NAME")
                        .takes_value(true)
                        .help("The friendly name of the save which will be updated"),
                )
                .arg(
                    Arg::with_name("path")
                        .help("The path of the save which will be updated")
                        .index(1),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("add", Some(sub_matches)) => add_save(sub_matches),
        ("delete", Some(sub_matches)) => del_path(sub_matches),
        ("info", Some(sub_matches)) => get_info(sub_matches),
        ("list", Some(_sub_matches)) => list_tracked(),
        ("update", Some(sub_matches)) => update_saves(sub_matches),
        _ => {}
    }
}

// Maybe move these functions into a separate module?
fn add_save(args: &ArgMatches) {
    let _manager = ConfigManager::default();
    let config = Config::static_config();

    dbg!(&config.db_location);

    match args.value_of("path") {
        Some(path) => {
            use cli::archive::options::SaveOptions;

            let username = (&config.local_username).clone();
            let db = Database::new(&config.db_location);
            let user = get_local_user(&db, &username);
            let path = PathBuf::from(path);
            let mut opt = SaveOptions {
                friendly_name: None,
            };

            if let Some(name) = args.value_of("friendly") {
                opt.friendly_name = Some(name)
            }

            Archive::create_save(&db, &user, &path, opt).expect("Unable to create Save");
        }
        None => eprintln!("No save path was provided."),
    }
}

fn del_path(_args: &ArgMatches) {
    unimplemented!()
}

fn get_info(_args: &ArgMatches) {
    unimplemented!()
}

fn list_tracked() {
    unimplemented!()
}

fn update_saves(_args: &ArgMatches) {
    unimplemented!()
}

fn get_local_user(db: &Database, username: &str) -> User {
    use chrono::Utc;
    use save_sync::archive::query::UserQuery;

    let query = UserQuery::new().with_username(&username);
    let option = db.get_user(query);

    match option {
        Some(user) => user,
        None => {
            // No user found. Is this the first time save sync is being run, or has the user changed?
            let potential_users = db.get_all_users();
            match potential_users {
                Some(users) => {
                    if users.len() == 1 {
                        // There is only one user in the DB. We can assume that this is the new default.
                        let old_config = Config::clone_config();
                        let new_default_user = users.first().unwrap();
                        let local_username = new_default_user.username.clone();

                        let new_config = Config {
                            local_username,
                            ..old_config
                        };
                        Config::update(new_config);

                        let manager = ConfigManager::default();
                        manager.write_to_file(); // Update the Config File

                        new_default_user.clone()
                    } else {
                        // TODO: Implement asking the user which profile they would like to migrate all their saves to.
                        todo!();
                    }
                }
                None => {
                    // This is the first time Save Sync is being run. We can generate a new user.
                    let time = Utc::now().naive_utc();

                    let new_user = NewUser {
                        username: &username,
                        created_at: time,
                        modified_at: time,
                    };

                    db.create_user(new_user);

                    let query = UserQuery::new().with_username(&username);
                    db.get_user(query).expect(
                        "Despite just writing the user to db, Save Sync was unable to retrieve it.",
                    )
                }
            }
        }
    }
}
