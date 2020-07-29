use clap::{crate_version, App, Arg};
use log::{error, info};
use std::{fs::read, process};

mod parser;
mod scan;

fn main() {
    env_logger::init();
    let matches = App::new("Repo Manifest Generator")
        .arg(
            Arg::with_name("config")
                .short("c")
                .required(true)
                .value_name("FILE")
                .help("Specify the configuration file to use"),
        )
        .version(crate_version!())
        .get_matches();
    let config = matches.value_of("config").unwrap();
    info!("Reading config from {}...", config);
    let config_data = read(config);
    if let Err(e) = config_data {
        error!("Could not read the config file {}: {}", config, e);
        process::exit(1);
    }
    let config_data = parser::parse_config(&config_data.unwrap());
    if let Err(e) = config_data {
        error!("Could not parse the config file {}: {}", config, e);
        process::exit(1);
    }
    let config_data = config_data.unwrap();
    info!("Preflight scanning...");
    let files = scan::collect_files(parser::get_root_path(&config_data));
    if let Err(e) = files {
        error!("Could not scan the directory: {}", e);
        process::exit(1);
    }
    let files = files.unwrap();
    if files.len() < 1 {
        error!("No tarball was found.");
        process::exit(1);
    }
    scan::scan_files(&files);
}
