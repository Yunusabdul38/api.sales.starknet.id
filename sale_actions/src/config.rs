use serde::{self, Deserialize};
use std::env;
use std::fs;

pub_struct!(Clone, Deserialize; General {
    check_delay: u64,
});

pub_struct!(Clone, Deserialize; Email {
    base_url : String,
    api_key: String,
    ar_group_id : String,
    batch_size : usize,
});

pub_struct!(Clone, Deserialize; Database {
    name: String,
    connection_string: String,
});

pub_struct!(Clone, Deserialize; WatchtowerTypes {
    info: String,
    warning: String,
    severe: String,
});

pub_struct!(Clone, Deserialize; Watchtower {
    enabled : bool,
    endpoint: String,
    app_id: String,
    token: String,
    types: WatchtowerTypes,
});

pub_struct!(Clone, Deserialize;  Config {
    general : General,
    email : Email,
    database: Database,
    watchtower: Watchtower,
});

pub fn load() -> Config {
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() <= 1 {
        "config.toml"
    } else {
        args.get(1).unwrap()
    };
    let file_contents = fs::read_to_string(config_path);
    if file_contents.is_err() {
        panic!("error: unable to read file with path \"{}\"", config_path);
    }

    match toml::from_str(file_contents.unwrap().as_str()) {
        Ok(loaded) => loaded,
        Err(err) => {
            panic!("error: unable to deserialize config. {}", err);
        }
    }
}
