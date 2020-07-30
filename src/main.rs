use clap::{crate_version, App, Arg};
use log::{error, info};
use std::{
    fs::{create_dir_all, read, write},
    path::Path,
    process,
};

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
    let root_path = parser::get_root_path(&config_data);
    let files = scan::collect_files(&root_path);
    if let Err(e) = files {
        error!("Could not scan the directory: {}", e);
        process::exit(1);
    }
    let files = files.unwrap();
    if files.len() < 1 {
        error!("No tarball was found.");
        process::exit(1);
    }
    info!("Scanning {} tarballs...", files.len());
    let files = scan::scan_files(&files, &root_path);
    if let Err(e) = files {
        error!("Could not scan the directory: {}", e);
        process::exit(1);
    }
    let files = files.unwrap();
    info!("Generating manifest...");
    let variants = parser::assemble_variants(&config_data, files);
    let manifest = parser::assemble_manifest(config_data, variants);
    let json = parser::generate_manifest(&manifest);
    if let Err(e) = json {
        error!("Could not generate the manifest: {}", e);
        process::exit(1);
    }
    info!("Writing manifest...");
    let manifest_dir = Path::new(&root_path).join("manifest");
    if let Err(e) = create_dir_all(&manifest_dir) {
        error!("Could not create directory: {}", e);
        process::exit(1);
    }
    if let Err(e) = write(manifest_dir.join("recipe.json"), json.unwrap()) {
        error!("Could not write the manifest: {}", e);
        process::exit(1);
    }
    info!("Manifest generated successfully.");
}
