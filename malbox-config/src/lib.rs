use serde::Deserialize;
use std::fs;
use std::process::exit;

#[derive(Deserialize)]
struct Config {
    http: Http,
    postgres: Postgres,
    debug: Debug,
}

#[derive(Deserialize)]
struct Http {
    host: String,
    port: u16,
}

#[derive(Deserialize)]
struct Postgres {
    database_url: String,
    port: u16,
}

#[derive(Deserialize)]
struct Debug {
    rust_log: String,
}

pub fn load_config() {
    let file_name = "../malbox.toml";

    let contents = match fs::read_to_string(file_name) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read file `{}`", file_name);
            exit(1);
        }
    };

    let data: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Unable to load data from file");
            exit(1);
        }
    };

    println!("{}", data.http.host);
}
