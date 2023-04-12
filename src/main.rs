extern crate core;
#[macro_use]
extern crate self_update;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::{env, process};
use std::env::temp_dir;
use std::string::String;
use std::time::Instant;

use clap::{ArgAction, ArgMatches};
use colored::Colorize;

use crate::api::spigotmc::SpigotAPI;
use crate::backup::BackupFormat;

mod api;
mod hashutils;
mod githubutils;
mod server_jars_com;
mod backup;
mod number_utils;

static mut SUB_COMMAND_ARG_MATCHES: Option<ArgMatches> = None;

fn show_example() {
    println!("{} {} {}", format!("Something went wrong!").red().bold(), format!("Example:").yellow(), format!("./limonium download paper 1.19.4").green());
}

#[tokio::main]
async fn main() {
    let matches_commands = clap::Command::new("limonium")
        .version(cargo_crate_version!())
        .author("Andrew121410")
        .about("A tiny Minecraft Server management tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(clap::Arg::new("self-update")
            .help("Updates Limonium")
            .long("self-update")
            .global(true)
            .action(ArgAction::SetTrue)
            .required(false))
        .subcommand(clap::Command::new("download")
            .about("Downloads a server jar")
            .arg(clap::Arg::new("software")
                .help("The software to download (paper, spigot, etc)")
                .action(ArgAction::Set)
                .required(true)
                .index(1))
            .arg(clap::Arg::new("version")
                .help("The version of the server to download")
                .action(ArgAction::Set)
                .required(true)
                .index(2))
            .arg(clap::Arg::new("path")
                .help("The path to download the server to (example: server.jar or ./server.jar or ./servers/hub/server.jar)")
                .short('o')
                .long("output")
                .aliases(["o", "n", "name"])
                .action(ArgAction::Set)
                .required(false))
            .arg(clap::Arg::new("serverjars.com")
                .help("Downloads the server from serverjars.com")
                .long("serverjars.com")
                .action(ArgAction::SetTrue)
                .required(false))
            .arg(clap::Arg::new("channel")
                .help("Choose the server to download [Example for Geyser the default is \"standalone\" the choices are (spigot, bungeecord, standalone, velocity, ...)]")
                .short('c')
                .aliases(["c"])
                .action(ArgAction::Set)
                .required(false))
            .arg(clap::Arg::new("latest-use-at-your-own-risk")
                .help("Downloads the latest version of the server (use at your own risk)")
                .long("latest-use-at-your-own-risk")
                .action(ArgAction::SetTrue)
                .required(false))
            .arg(clap::Arg::new("latest-dont-include-snapshot-versions")
                .help("When searching for the latest version of the server, don't include snapshot versions")
                .long("latest-dont-include-snapshot-versions")
                .action(ArgAction::SetTrue)
                .required(false)))
        .subcommand(clap::Command::new("backup")
            .about("Backs up the server")
            .arg(clap::Arg::new("name")
                .help("The name of the backup")
                .action(ArgAction::Set)
                .required(true)
                .index(1))
            .arg(clap::Arg::new("to_backup")
                .help("The path to backup")
                .action(ArgAction::Set)
                .required(true)
                .index(2))
            .arg(clap::Arg::new("backup_folder")
                .help("The folder to backup to")
                .action(ArgAction::Set)
                .required(true)
                .index(3))
            .arg(clap::Arg::new("exclude")
                .help("The files to exclude from the backup")
                .action(ArgAction::Set)
                .required(false))
            .arg(clap::Arg::new("format")
                .help("The format to backup to")
                .action(ArgAction::Set)
                .required(false)
                .default_value("tar.gz")
                .value_parser(["zip", "tar.gz"])));

    let command_matches: ArgMatches = matches_commands.get_matches();

    // Handle self-update
    if command_matches.get_flag("self-update") {
        if self_update() {
            process::exit(0); // Exit if updated
            return;
        }
    }

    match command_matches.subcommand() {
        Some(("download", download_matches)) => {
            unsafe { SUB_COMMAND_ARG_MATCHES = Some(download_matches.clone()); }
            handle_download(&download_matches).await;
        }
        Some(("backup", backup_matches)) => {
            unsafe { SUB_COMMAND_ARG_MATCHES = Some(backup_matches.clone()); }
            handle_backup(&backup_matches);
        }
        _ => {
            show_example();
            process::exit(1);
        }
    }
}

async fn handle_download(download_matches: &ArgMatches) {
    let current_dir_path_buffer = env::current_dir().unwrap();
    let current_path = current_dir_path_buffer.as_path();

    let software = download_matches.get_one::<String>("software").unwrap();
    let mut version: String = download_matches.get_one::<String>("version").unwrap().clone();


    let latest_use_at_your_own_risk = download_matches.get_flag("latest-use-at-your-own-risk");
    // Test to see if version is "latest
    if version.eq_ignore_ascii_case("latest") && !latest_use_at_your_own_risk {
        println!("{} {} {} {}", format!("Something went wrong!").red().bold(), format!("Using").yellow(), format!("latest").red(), format!("is not recommended!").yellow());
        println!("{} {}", format!("Use").yellow(), format!("--latest-use-at-your-own-risk").red());
        print!("");
        println!("{}", format!("This is because you don't want your Minecraft Server randomly getting updated to a new Minecraft version without you knowing!").yellow());
        process::exit(102);
    }

    let temp = String::from("");
    let mut path_string = download_matches.get_one::<String>("path").unwrap_or(&temp).to_string();

    // Handle serverjars.com flag
    let use_serverjars_com = download_matches.get_flag("serverjars.com");
    if use_serverjars_com {
        println!("{} {}", format!("Downloading from").yellow(), format!("serverjars.com").red());
        server_jars_com::download_jar(&software, &version, &mut path_string).await;
        return; // Don't continue
    }

    // Check if the software is supported
    if !api::is_valid_platform(&software) {
        println!("{} {} {} {}", format!("Something went wrong!").red().bold(), format!("Project").yellow(), format!("{}", &software).red(), format!("is not valid!").yellow());
        process::exit(102);
    }

    // Handle SpigotMC
    if software.eq_ignore_ascii_case("spigot") {
        if path_string.eq("") {
            path_string.push_str("./spigot-");
            path_string.push_str(&version);
            path_string.push_str(".jar");
        }

        SpigotAPI::download_build_tools();
        SpigotAPI::run_build_tools(&version, &path_string);
        return; // Don't continue
    }

    let platform = api::get_platform(&software);

    // Get the latest version if the version is "latest" (use at your own risk)
    if version.eq_ignore_ascii_case("latest") {
        let latest_version: Option<String> = platform.get_latest_version(&software).await;

        if latest_version.is_none() {
            println!("{} {}", format!("Something went wrong!").red().bold(), format!("Couldn't get the latest version!").yellow());
            println!("{}", format!("This is most likely because the platform doesn't support getting the latest version!").yellow());

            process::exit(102);
        }

        version = latest_version.unwrap();
    }

    let build = platform.get_latest_build(&software, &version).await.expect("Getting the latest build failed?");

    // Set the path if it's empty
    if path_string.eq("") {
        path_string.push_str(platform.get_jar_name(&software, &version, &build).as_str());
    }

    // Start elapsed time
    let start = Instant::now();

    // Get the hash of the jar from a API
    let hash_optional = platform.get_jar_hash(&software, &version, &build).await;

    // Verify if we need to download the jar by checking the hash of the current installed jar
    if hash_optional.is_some() {
        let hash = hash_optional.as_ref().unwrap();

        if current_path.join(&path_string).exists() {
            let does_match = hashutils::validate_the_hash(&hash, &current_path, &path_string, false);
            if does_match {
                // Don't download the jar if the hash is the same
                println!("{} {} {}", format!("You are already up to date!").green().bold(), format!("Path:").yellow(), format!("{}", &path_string).blue().bold());
                return;
            }
        }
    }

    let tmp_jar_name = api::download_jar_to_temp_dir(&platform.get_download_link(&software, &version, &build)).await;

    // Verify the hash of the downloaded jar in the temp directory
    if hash_optional.is_some() {
        let hash = &hash_optional.unwrap();
        hashutils::validate_the_hash(&hash, &temp_dir(), &tmp_jar_name, true);
    } else {
        println!("{}", format!("Not checking hash!").yellow().bold());
    }

    api::copy_jar_from_temp_dir_to_dest(&tmp_jar_name, &path_string);

    let duration = start.elapsed().as_millis().to_string();
    println!("{} {} {} {}", format!("Downloaded JAR:").green().bold(), format!("{}", &path_string.as_str()).blue().bold(), format!("Time In Milliseconds:").purple().bold(), format!("{}", &duration).yellow().bold());
}

fn handle_backup(backup_matches: &ArgMatches) {
    let current_dir_path_buffer = env::current_dir().unwrap();
    let current_path = current_dir_path_buffer.as_path();

    let name = backup_matches.get_one::<String>("name").unwrap();
    let to_backup = backup_matches.get_one::<String>("to_backup").unwrap();
    let backup_folder = backup_matches.get_one::<String>("backup_folder").unwrap();
    let format = backup_matches.get_one::<String>("format").unwrap();
    let exclude: Option<&String> = backup_matches.get_one::<String>("exclude");

    // Lazy to mess with lifetimes so I'm just going to do this Lol..
    let mut exclude_ours: Option<String> = None;
    match exclude {
        Some(string) => {
            exclude_ours = Some(string.to_string());
        }
        _ => {}
    }

    let mut backup_format: BackupFormat = BackupFormat::TarGz;
    if format.eq("zip") {
        backup_format = BackupFormat::Zip;
    }

    let backup_folder_pathbuf = current_path.join(backup_folder);
    let backup = backup::Backup::new(name.to_string(), to_backup.to_string(), backup_folder_pathbuf, backup_format, exclude_ours);

    let time = Instant::now();

    // If error show error
    let the_backup = backup.backup();
    if the_backup.is_err() {
        println!("{} {} {}", format!("Something went wrong!").red().bold(), format!("Error:").yellow(), format!("{}", the_backup.err().unwrap()).red());
        process::exit(102);
    }

    let time_elapsed_seconds = time.elapsed().as_secs();
    if time_elapsed_seconds > 65 {
        let time_elapsed_minutes = time_elapsed_seconds / 60;
        println!("{} {} {}", format!("Backup completed!").green().bold(), format!("Time elapsed:").yellow(), format!("{} minutes", time_elapsed_minutes).green());
    } else {
        println!("{} {} {}", format!("Backup completed!").green().bold(), format!("Time elapsed:").yellow(), format!("{} seconds", time_elapsed_seconds).green());
    }
}

fn self_update() -> bool {
    println!("Current Version: {}", cargo_crate_version!());
    let status = self_update::backends::github::Update::configure()
        .repo_owner("andrew121410")
        .repo_name("limonium")
        .target("limonium-x86_64-unknown-linux-gnu.zip")
        .bin_name("limonium")
        .no_confirm(true)
        .show_download_progress(false)
        .show_output(false)
        .current_version(cargo_crate_version!())
        .build().expect("Failed to build update")
        .update().expect("Failed to update");
    return if status.updated() {
        println!("Updated Limonium from {} to {}", cargo_crate_version!(), &status.version());
        true
    } else {
        println!("Limonium is already up to date!");
        false
    };
}