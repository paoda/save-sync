use clap::{App, Arg, ArgMatches, SubCommand};
use cli::archive::Archive;
use save_sync::archive::query::{SaveQuery, UserQuery};
use save_sync::config::Config;
use save_sync::models::{NewUser, Save, User};
use save_sync::ConfigManager;
use save_sync::Database;
use std::path::PathBuf;

fn main() {
    let _manager = ConfigManager::default(); // Initialize Config

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
        .subcommand(
            SubCommand::with_name("verify")
                .about("Verify that Save Backup is up to date.")
                .arg(
                    Arg::with_name("friendly")
                        .short("f")
                        .long("friendly")
                        .value_name("NAME")
                        .takes_value(true)
                        .help("The friendly name of the save that you want to verify"),
                )
                .arg(
                    Arg::with_name("path")
                        .help("The path of the save that you want to verify.")
                        .index(1)
                        .required_unless("friendly"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("add", Some(sub_matches)) => add_save(sub_matches),
        ("delete", Some(sub_matches)) => del_save(sub_matches),
        ("info", Some(sub_matches)) => get_save_info(sub_matches),
        ("list", Some(_sub_matches)) => list_tracked_saves(),
        ("update", Some(sub_matches)) => update_saves(sub_matches),
        ("verify", Some(sub_matches)) => verify_save(sub_matches),
        _ => {}
    }
}

// Maybe move these functions into a separate module?
fn add_save(args: &ArgMatches) {
    use cli::archive::options::SaveOptions;

    let config = Config::static_config();
    let path = args.value_of("path").unwrap(); // required

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

fn del_save(args: &ArgMatches) {
    let config = Config::static_config();
    let db = Database::new(&config.db_location);
    let save: Save;

    if let Some(name) = args.value_of("friendly") {
        let query = SaveQuery::new().with_friendly_name(name);
        let option = db.get_save(query);

        match option {
            Some(result) => save = result,
            None => eprintln!("{} is not related to any save in the database.", name),
        }
    } else {
        let path = args.value_of("path").unwrap(); // Required if friendly is not set
        let query = SaveQuery::new().with_path(PathBuf::from(path));
        let option = db.get_save(query);

        match option {
            Some(result) => save = result,
            None => eprintln!("{} is not a tracked save path in the database.", path),
        }
    }

    unimplemented!()
}

fn get_save_info(args: &ArgMatches) {
    let config = Config::static_config();
    let db = Database::new(&config.db_location);
    let mut save: Option<Save> = None;

    if let Some(name) = args.value_of("friendly") {
        // Get save by friendly name.
        let query = SaveQuery::new().with_friendly_name(name);
        let option = db.get_save(query);

        match option {
            Some(result) => save = Some(result),
            None => eprintln!("There was no save labelled as \"{}\" in the db.", name),
        }
    } else {
        let path = args.value_of("path").unwrap(); // Required if friendly is not set
                                                   // get save by save path.
        let query = SaveQuery::new().with_path(PathBuf::from(path));
        let option = db.get_save(query);

        match option {
            Some(result) => save = Some(result),
            None => eprintln!(
                "\"{}\" is not a path which is stored in the database.",
                path
            ),
        }
    }

    if let Some(save) = save {
        println!("\"{}\"", save.save_path);
        println!("---");

        if save.friendly_name.is_empty() {
            println!("Friendly name: none");
        } else {
            println!("Friendly name: {}", save.friendly_name);
        }

        // Get user which owns this save.
        let query = UserQuery::new().with_id(save.user_id);
        let option = db.get_user(query);

        match option {
            Some(user) => println!("Belongs to: {}", user.username),
            None => println!("Belongs to: User #{}", save.user_id),
        }

        println!("UUID: {}", save.uuid);
        println!("Backup path: {}", save.backup_path);
        println!("Created: {}", save.created_at);
        println!("Modified: {}", save.modified_at);
    }
}

fn list_tracked_saves() {
    let config = Config::static_config();
    let db = Database::new(&config.db_location);
    let user = get_local_user(&db, &config.local_username);

    let query = SaveQuery::new().with_user_id(user.id);
    let option = db.get_saves(query);

    match option {
        Some(saves) => {
            for save in saves {
                let friendly_name = save.friendly_name;
                let save_path = save.save_path;
                let uuid = save.uuid;

                if !friendly_name.is_empty() {
                    print!("[{}]: ", friendly_name);
                }

                println!("\"{}\" | {{{}}}", save_path, uuid);
            }
        }
        None => eprintln!("No saves in database."),
    }
}

fn verify_save(args: &ArgMatches) {
    let config = Config::static_config();
    let db = Database::new(&config.db_location);
    let mut save: Option<Save> = None;

    if let Some(name) = args.value_of("friendly") {
        // Get save by friendly name.
        let query = SaveQuery::new().with_friendly_name(name);
        let option = db.get_save(query);

        match option {
            Some(result) => save = Some(result),
            None => eprintln!("There was no save labelled as \"{}\" in the db.", name),
        }
    } else {
        let path = args.value_of("path").unwrap(); // Required unless friendly is set.
        let query = SaveQuery::new().with_path(PathBuf::from(path));
        let option = db.get_save(query);

        match option {
            Some(result) => save = Some(result),
            None => eprintln!(
                "\"{}\" is not a path which is stored in the database.",
                path
            ),
        }
    }

    if let Some(save) = save {
        let (new_files, changed_files) =
            Archive::verify_save(&db, &save).expect("Unable to Verify Integrity of Save");

        if new_files.is_empty() && changed_files.is_empty() {
            if save.friendly_name.is_empty() {
                println!("No changed were detected in {}", save.save_path)
            } else {
                println!("{}'s backup is up to date.", save.friendly_name)
            }
        } else {
            println!("The Backup and the current save differ");
            println!();

            if !new_files.is_empty() {
                println!("New Files:");
                for file in new_files {
                    println!("{}", file.to_string_lossy());
                }
            }

            if !changed_files.is_empty() {
                println!("Changed Files:");
                for file in changed_files {
                    println!("{}", file.to_string_lossy());
                }
            }
        }
    }
}

fn update_saves(_args: &ArgMatches) {
    unimplemented!()
}

fn get_local_user(db: &Database, username: &str) -> User {
    use chrono::Utc;

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
