extern crate core;
#[macro_use]
extern crate self_update;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use crate::backup::BackupFormat;
use crate::log_search::LogSearch;
use crate::objects::DownloadedJar::DownloadedJar;
use clap::builder::TypedValueParser;
use clap::{ArgAction, ArgMatches};
use colored::Colorize;
use std::env::temp_dir;
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::string::String;
use std::time::Instant;
use std::{env, fs, process};

mod backup;
mod clap_utils;
mod compile_controllers;
mod download_controllers;
mod github_utils;
mod hash_utils;
mod jenkins_utils;
mod log_search;
mod number_utils;
mod objects;
mod ensurer;
mod file_utils;
mod jvm_downgrader;

fn show_example() {
    println!(
        "{} {} {}",
        format!("Something went wrong!").red().bold(),
        format!("Example:").yellow(),
        format!("./limonium download paper 1.21.5").green()
    );
}

fn print_banner() {
    let title = "Limonium";
    let subtitle = "A tiny Minecraft Server management tool";
    let version = format!("Version: {}", cargo_crate_version!());
    let developer = "Developed by Andrew121410!";

    // Determine the width of the box based on the longest line
    let width = usize::max(
        title.len(),
        usize::max(subtitle.len(), usize::max(version.len(), developer.len())),
    ) + 4; // Adding padding for border

    // Border
    let border = "*".repeat(width);

    // Print the box with content centered
    println!("{}", border.green().bold());
    println!(
        "*{}*",
        format!(
            "{}{}{}",
            " ".repeat((width - title.len() - 2) / 2),
            title.green().bold(),
            " ".repeat((width - title.len() - 2) / 2)
        )
    );
    println!("*{}*", " ".repeat(width - 2)); // Empty line
    println!(
        "*{}*",
        format!(
            "{}{}{}",
            " ".repeat((width - subtitle.len() - 2) / 2),
            subtitle.yellow(),
            " ".repeat((width - subtitle.len() - 2) / 2)
        )
    );
    println!("*{}*", " ".repeat(width - 2)); // Empty line
    println!(
        "*{}*",
        format!(
            "{}{}{}",
            " ".repeat((width - version.len() - 2) / 2),
            version.green(),
            " ".repeat((width - version.len() - 2) / 2)
        )
    );
    println!("*{}*", " ".repeat(width - 2)); // Empty line
    println!(
        "*{}*",
        format!(
            "{}{}{}",
            " ".repeat((width - developer.len() - 2) / 2),
            developer.cyan(),
            " ".repeat((width - developer.len() - 2) / 2)
        )
    );
    println!("{}", border.green().bold());
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
        .arg(clap::Arg::new("no-banner")
            .short('b')
            .long("nb")
            .aliases(["b"]) // --b
            .help("Do not display the cool banner when program starts")
            .action(ArgAction::SetTrue))
        .subcommand(clap::Command::new("self-update")
            .about("Updates Limonium"))
        .subcommand(clap::Command::new("compile")
            .about("Compiles software")
            .arg(clap::Arg::new("software")
                .help("The software to compile (paper, spigot, etc)")
                .action(ArgAction::Set)
                .required(true)
                .index(1))
            .arg(clap::Arg::new("path")
                .help("The path to compile the software to (example: server.jar or ./server.jar or ./servers/hub/server.jar)")
                .short('o')
                .long("output")
                .aliases(["o", "n", "name"])
                .action(ArgAction::Set)
                .index(2)
                .required(true))
            .arg(clap::Arg::new("version")
                .long("version")
                .help("The version of the software to compile")
                .action(ArgAction::Set)
                .required(false))
            .arg(clap::Arg::new("branch")
                .help("The branch to compile")
                .short('b')
                .long("branch")
                .aliases(["b"])
                .action(ArgAction::Set)
                .required(false)))
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
            .arg(clap::Arg::new("no-snapshot-version")
                .help("When searching for the latest version of the server, don't include snapshot versions")
                .long("no-snapshot-version")
                .action(ArgAction::SetTrue)
                .required(false))
            .arg(clap::Arg::new("run-jvmdowngrader")
                .help("After downloading the server, run the JAR through the JVM Downgrader")
                .long("run-jvmdowngrader")
                .action(ArgAction::Set)
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
                .help("The format to use (tar.gz, tar.zst, zip)")
                .long("format")
                .action(ArgAction::Set)
                .required(false)
                .default_value("tar.gz")
                .value_parser(["tar.gz", "tar.zst", "zip"]))
            .arg(clap::Arg::new("level")
                .help("The compression level to use")
                .long("level")
                .action(ArgAction::Set)
                .required(false)
                .value_parser(clap::value_parser!(i64)))
            .arg(clap::Arg::new("sftp")
                .help("The SFTP server to backup to")
                .long("sftp")
                .action(ArgAction::Set)
                .required(false))
            .arg(clap::Arg::new("delete-after-upload")
                .help("Deletes the backup after uploading it")
                .long("delete-after-upload")
                .action(ArgAction::SetTrue)
                .required(false))
            .arg(clap::Arg::new("local-delete-after-time")
                .help("Deletes backups after a certain amount of time LOCALLY")
                .long("local-delete-after-time")
                .action(ArgAction::Set)
                .required(false))
            .arg(clap::Arg::new("local-always-keep")
                .help("Always keep a certain amount of backups LOCALLY")
                .long("local-always-keep")
                .action(ArgAction::Set)
                .required(false))
            .arg(clap::Arg::new("remote-delete-after-time")
                .help("Deletes backups after a certain amount of time REMOTELY")
                .long("remote-delete-after-time")
                .action(ArgAction::Set)
                .required(false))
            .arg(clap::Arg::new("ask-before-uploading")
                .help("Ask if you want to upload the backup to SFTP before uploading")
                .long("ask-before-uploading")
                .action(ArgAction::SetTrue)
                .required(false))
            .arg(clap::Arg::new("verbose")
                .help("Shows more information")
                .long("verbose")
                .short('v')
                .default_value("false")
                .action(ArgAction::SetTrue)
                .required(false))
            .arg(clap::Arg::new("I")
                .help("Overrides -I for tar")
                .long("I")
                .short('I')
                .action(ArgAction::Set)
                .required(false)))
        .subcommand(clap::Command::new("log")
            .about("Searches the server logs folder for a string")
            .arg(clap::Arg::new("days-back")
                .help("The amount of days back to search")
                .value_parser(clap::value_parser!(u64))
                .action(ArgAction::Set)
                .required(true)
                .index(1))
            .arg(clap::Arg::new("search")
                .help("The string to search for")
                .action(ArgAction::Set)
                .required(true)
                .index(2))
            .arg(clap::Arg::new("lines-before")
                .help("The amount of lines before the search to show")
                .value_parser(clap::value_parser!(u64))
                .requires("lines-after")
                .action(ArgAction::Set)
                .required(false)
                .index(3))
            .arg(clap::Arg::new("lines-after")
                .help("The amount of lines after the search to show")
                .value_parser(clap::value_parser!(u64))
                .requires("lines-before")
                .action(ArgAction::Set)
                .required(false)
                .index(4))
            .arg(clap::Arg::new("path")
                .help("The path to the log folder")
                .short('p')
                .long("path")
                .aliases(["p"])
                .action(ArgAction::Set)
                .required(false)
                .default_value("./logs")));

    let command_matches: ArgMatches = matches_commands.get_matches();

    // Do not display the cool box if it's passed.
    if !command_matches.get_flag("no-banner") {
        print_banner();
        println!();
        println!();
    }

    // Handle self-update flag
    if command_matches.get_flag("self-update") {
        if self_update() {
            process::exit(0); // Exit if updated
        }
    }

    // Cleanup temp directory
    if file_utils::get_limonium_dir().exists() {
        println!("{}", "Cleaning up /tmp/limonium folder...".yellow());
        file_utils::delete_limonium_folder().unwrap();
        println!("{}", "Done! Cleaning up /tmp/limonium folder...".green());
    }

    match command_matches.subcommand() {
        // Handle self-update subcommand
        Some(("self-update", _)) => {
            self_update();
            process::exit(0);
        }
        Some(("download", download_matches)) => {
            // Set the subcommand arg matches
            clap_utils::write_sub_command_arg_matches(download_matches.clone());

            handle_download(&download_matches).await;
        }
        Some(("compile", compile_matches)) => {
            // Set the suncommand arg matches
            clap_utils::write_sub_command_arg_matches(compile_matches.clone());

            compile_controllers::CompileController::handle_compile(&compile_matches).await;
        }
        Some(("backup", backup_matches)) => {
            // Set the subcommand arg matches
            clap_utils::write_sub_command_arg_matches(backup_matches.clone());

            handle_backup(&backup_matches).await;
        }
        Some(("log", log_matches)) => {
            // Set the subcommand arg matches
            clap_utils::write_sub_command_arg_matches(log_matches.clone());

            handle_log_search(&log_matches).await;
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
    let mut version: String = download_matches
        .get_one::<String>("version")
        .unwrap()
        .clone();

    let latest_use_at_your_own_risk = download_matches.get_flag("latest-use-at-your-own-risk");
    // Test to see if version is "latest
    if version.eq_ignore_ascii_case("latest") && !latest_use_at_your_own_risk {
        println!(
            "{} {} {} {}",
            format!("Something went wrong!").red().bold(),
            format!("Using").yellow(),
            format!("latest").red(),
            format!("is not recommended!").yellow()
        );
        println!(
            "{} {}",
            format!("Use").yellow(),
            format!("--latest-use-at-your-own-risk").red()
        );
        print!("");
        println!("{}", format!("This is because you don't want your Minecraft Server randomly getting updated to a new Minecraft version without you knowing!").yellow());
        process::exit(102);
    }

    let temp = String::from("");
    let mut path_string = download_matches
        .get_one::<String>("path")
        .unwrap_or(&temp)
        .to_string();

    // Check if the software is supported
    if !download_controllers::is_valid_platform(&software) {
        println!(
            "{} {} {} {}",
            format!("Something went wrong!").red().bold(),
            format!("Project").yellow(),
            format!("{}", &software).red(),
            format!("is not valid!").yellow()
        );
        process::exit(102);
    }

    // Handle SpigotMC
    if software.eq_ignore_ascii_case("spigot") {
        // If you want to download Spigot you can't so you'll have to compile it
        // Like ./limonium compile spigot --version 1.21.1 --o server.jar
        println!(
            "{} {}",
            format!("Something went wrong!").red().bold(),
            format!("You can't download Spigot!").yellow()
        );
        println!("{}", format!("You'll have to compile it!").yellow());

        // Show example
        println!("{}", format!("Example:").yellow());
        println!(
            "{}",
            format!("./limonium compile spigot server.jar --version 1.21.5").green()
        );

        return; // Don't continue
    }

    let platform = download_controllers::get_platform(&software);

    // Get the latest version if the version is "latest" (use at your own risk)
    if version.eq_ignore_ascii_case("latest") {
        let latest_version: Option<String> = platform.get_latest_version(&software).await;

        if latest_version.is_none() {
            println!(
                "{} {}",
                format!("Something went wrong!").red().bold(),
                format!("Couldn't get the latest version!").yellow()
            );
            println!("{}", format!("This is most likely because the platform({}) doesn't support getting the latest version!", &software).yellow());

            process::exit(102);
        }

        version = latest_version.unwrap();
    }

    // Get the latest build for the version
    let build_option = platform
        .get_latest_build(&software, &version)
        .await;
    if build_option.is_none() {
        println!(
            "{} {}",
            format!("Something went wrong!").red().bold(),
            format!("Couldn't get the latest build!").yellow()
        );
        println!("{}", format!("This is most likely because that platform({}) has no build for that version({})", &software, &version).yellow());

        process::exit(102);
    }
    let build = build_option.unwrap();

    // Set the path if it's empty
    if path_string.eq("") {
        path_string.push_str(platform.get_jar_name(&software, &version, &build).as_str());
    }

    // Start elapsed time
    let start = Instant::now();

    // Get the hash of the jar from a API
    let hash_before_downloaded_jar = platform
        .get_hash_from_web(&software, &version, &build, None)
        .await;

    // Verify if we need to download the jar by checking the hash of the current installed jar
    if hash_before_downloaded_jar.is_some() {
        let hash = hash_before_downloaded_jar.as_ref().unwrap();

        if current_path.join(&path_string).exists() {
            let does_match =
                hash_utils::validate_the_hash(&hash, &current_path, &path_string, false);
            if does_match {
                // Don't download the jar if the hash is the same
                println!(
                    "{} {} {}",
                    format!("You are already up to date!").green().bold(),
                    format!("Path:").yellow(),
                    format!("{}", &path_string).blue().bold()
                );
                return;
            }
        }
    }

    let download_link = platform.get_download_link(&software, &version, &build);
    let mut downloaded_jar: DownloadedJar = DownloadedJar::empty();

    // Check if the platform has a custom download functionality
    let custom_download_function_result = platform
        .custom_download_functionality(&software, &version, &build, &download_link)
        .await;
    if custom_download_function_result.is_some() {
        downloaded_jar = custom_download_function_result.unwrap();
    } else {
        // If there's no custom download functionality, download the jar to the temp directory
        downloaded_jar =
            download_controllers::download_jar_to_temp_dir_with_progress_bar(&download_link).await;
    }

    // Verify the hash of the downloaded jar in the temp directory
    let hash_after_downloaded_jar = platform
        .get_hash_from_web(&software, &version, &build, Some(&downloaded_jar))
        .await;
    if hash_after_downloaded_jar.is_some() {
        let hash = &hash_after_downloaded_jar.unwrap();
        hash_utils::validate_the_hash(&hash, &file_utils::get_or_create_limonium_dir(), &downloaded_jar.temp_jar_name, true);
    } else {
        println!("{}", format!("Not checking hash!").yellow().bold());
    }

    // Run the JVM Downgrader if specified
    let run_jvmdowngrader = download_matches.get_one::<String>("run-jvmdowngrader");
    if run_jvmdowngrader.is_some() {
        let major_version = run_jvmdowngrader.unwrap().to_string();
        let input_jar = downloaded_jar.temp_jar_path;
        let output_jar = file_utils::get_or_create_limonium_dir().join(&downloaded_jar.temp_jar_name);

        jvm_downgrader::run_jvm_downgrader(&major_version, &input_jar, &output_jar).await;
    }

    // Copy the downloaded jar to the destination
    file_utils::copy_jar_from_temp_dir_to_dest(
        &downloaded_jar.temp_jar_name,
        &path_string,
    );

    let duration = start.elapsed().as_millis().to_string();
    println!(
        "{} {} {} {}",
        format!("Downloaded JAR:").green().bold(),
        format!("{}", &path_string.as_str()).blue().bold(),
        format!("Time In Milliseconds:").purple().bold(),
        format!("{}", &duration).yellow().bold()
    );
}

async fn handle_backup(backup_matches: &ArgMatches) {
    let current_dir_path_buffer = env::current_dir().unwrap();
    let current_path = current_dir_path_buffer.as_path();

    let name = backup_matches.get_one::<String>("name").unwrap();
    let to_backup = backup_matches.get_one::<String>("to_backup").unwrap();
    let backup_folder = backup_matches.get_one::<String>("backup_folder").unwrap();
    let format = backup_matches.get_one::<String>("format").unwrap();

    let exclude: Option<&String> = backup_matches.get_one::<String>("exclude");
    let mut exclude_ours: Option<String> = None;
    match exclude {
        Some(string) => {
            exclude_ours = Some(string.to_string());
        }
        _ => {}
    }

    let compression_level: Option<&i64> = backup_matches.get_one::<i64>("level");
    let mut compression_level_ours: Option<i64> = None;
    match compression_level {
        Some(level) => {
            compression_level_ours = Some(level.to_owned());
        }
        _ => {}
    }

    let mut backup_format: BackupFormat = BackupFormat::TarGz;
    if format.eq("tar.zst") {
        backup_format = BackupFormat::TarZst;
    } else if format.eq("zip") {
        backup_format = BackupFormat::Zip;
    }

    let backup_folder_pathbuf = current_path.join(backup_folder);
    let backup = backup::Backup::new(
        name.to_string(),
        to_backup.to_string(),
        backup_folder_pathbuf,
        backup_format,
        exclude_ours,
        compression_level_ours,
    );

    let time = Instant::now();

    // If error show error
    let the_backup = backup.backup();
    if the_backup.is_err() {
        println!(
            "{} {} {}",
            format!("Something went wrong!").red().bold(),
            format!("Error:").yellow(),
            format!("{}", the_backup.err().unwrap()).red()
        );
        process::exit(102);
    }

    let backup_result = the_backup.unwrap();

    // Handle deleting backups after a certain amount of time LOCALLY
    let local_delete_after_time = backup_matches.get_one::<String>("local-delete-after-time");
    let local_always_keep = backup_matches.get_one::<u64>("local-always-keep");
    if local_delete_after_time.is_some() {
        let local_delete_after_time_input = local_delete_after_time.unwrap().to_string();
        backup.local_delete_after_time(
            &local_delete_after_time_input,
            local_always_keep.map(|&v| v as usize),
        );

        println!(
            "{} {}",
            format!("Deleting LOCAL backups after").yellow(),
            format!("{}", local_delete_after_time_input).green()
        );
    }

    // Ask if you want to upload the backup to SFTP before uploading
    let ask_before_upload = backup_matches.get_flag("ask-before-uploading");
    let mut skip_upload = false;
    if ask_before_upload {
        skip_upload = ask_for_input_for_to_upload_to_sftp();
    }

    // Handle uploading to SFTP if SFTP is specified
    let sftp_option = backup_matches.get_one::<String>("sftp");
    if sftp_option.is_some() && !skip_upload {
        println!(
            "{} {}",
            format!("Uploading to SFTP!").green().bold(),
            format!("This may take a while depending on the size of the backup!").yellow()
        );

        let sftp_args = sftp_option.unwrap();
        let sftp_args_vector = sftp_args.split(" ").collect::<Vec<&str>>();
        let sftp_user_and_host_vector = sftp_args_vector[0].split("@").collect::<Vec<&str>>();

        // --sftp user@host:optional_port key_file remote_dir

        let sftp_user = sftp_user_and_host_vector[0];
        let mut sftp_host = sftp_user_and_host_vector[1];

        // Check if port is specified
        let mut sftp_port: Option<u16> = None;
        if sftp_host.contains(":") {
            let sftp_host_vector = sftp_host.split(":").collect::<Vec<&str>>();
            sftp_host = sftp_host_vector[0];
            sftp_port = Some(sftp_host_vector[1].parse::<u16>().unwrap());
        }

        // If sftp_args_vector length is 3, then we have a key file
        let mut sftp_key_file: Option<&Path> = None;
        if sftp_args_vector.len() == 3 {
            sftp_key_file = Some(Path::new(sftp_args_vector[1]));
        }

        let sftp_remote_dir = sftp_args_vector[sftp_args_vector.len() - 1];

        let result = backup
            .upload_sftp(
                sftp_user.to_string(),
                sftp_host.to_string(),
                sftp_port,
                sftp_key_file,
                &backup_result.file_path,
                backup_result.file_name,
                sftp_remote_dir.to_string(),
                (&backup_result.sha256_hash).to_string(),
            )
            .await;

        if result.is_err() {
            println!(
                "{} {} {}",
                format!("Something went wrong!").red().bold(),
                format!("Error:").yellow(),
                format!("{}", result.err().unwrap()).red()
            );
            process::exit(102);
        }

        // Handle deleting backups after a certain amount of time REMOTELY
        let remote_delete_after_time = backup_matches.get_one::<String>("remote-delete-after-time");
        if remote_delete_after_time.is_some() {
            let remote_delete_after_time_input = remote_delete_after_time.unwrap().to_string();
            backup
                .sftp_delete_after_time(
                    &remote_delete_after_time_input,
                    sftp_user.to_string(),
                    sftp_host.to_string(),
                    sftp_port,
                    sftp_key_file,
                    sftp_remote_dir.to_string(),
                )
                .await;

            println!(
                "{} {}",
                format!("Deleting REMOTE backups after").yellow(),
                format!("{}", remote_delete_after_time_input).green()
            );
        }

        // Handle deleting the file after upload if specified
        let delete_after_upload = backup_matches.get_flag("delete-after-upload");
        if delete_after_upload {
            let file_to_delete = backup_result.file_path;
            let result = fs::remove_file(file_to_delete);
            println!(
                "{} {}",
                format!("Deleting file after upload!").green().bold(),
                format!("File:").yellow()
            );
            if result.is_err() {
                println!(
                    "{} {} {}",
                    format!("Something went wrong!").red().bold(),
                    format!("Error:").yellow(),
                    format!("{}", result.err().unwrap()).red()
                );
                process::exit(102);
            }
        }
    } else if sftp_option.is_some() && skip_upload {
        println!(
            "{} {}",
            format!("Skipping upload to SFTP!").green().bold(),
            format!("Skipping upload to SFTP!").yellow()
        );
    }

    let time_elapsed_seconds = time.elapsed().as_secs();
    if time_elapsed_seconds > 65 {
        let time_elapsed_minutes = time_elapsed_seconds / 60;
        println!(
            "{} {} {}",
            format!("Backup completed!").green().bold(),
            format!("Time elapsed:").yellow(),
            format!("{} minutes", time_elapsed_minutes).green()
        );
    } else {
        println!(
            "{} {} {}",
            format!("Backup completed!").green().bold(),
            format!("Time elapsed:").yellow(),
            format!("{} seconds", time_elapsed_seconds).green()
        );
    }
}

async fn handle_log_search(log_search: &ArgMatches) {
    let days_back = log_search.get_one::<u64>("days-back").unwrap();
    let to_search = log_search.get_one::<String>("search").unwrap();
    let lines_before_option = log_search.get_one::<u64>("lines-before");
    let lines_after_option = log_search.get_one::<u64>("lines-after");
    let path = log_search.get_one::<String>("path").unwrap();

    println!(
        "{} {}",
        format!("Searching logs!").green().bold(),
        format!("This may take a while depending on the size of the logs!").yellow()
    );

    let logs_folder: PathBuf = PathBuf::from(path);
    let log_search: LogSearch =
        LogSearch::new(days_back.clone(), to_search.to_string(), logs_folder);
    if lines_before_option.is_none() && lines_after_option.is_none() {
        log_search.context(0, 0);
    } else if lines_before_option.is_some() && lines_after_option.is_some() {
        let lines_before = lines_before_option.unwrap();
        let lines_after = lines_after_option.unwrap();

        log_search.context(lines_before.clone(), lines_after.clone());
    }
}

fn ask_for_input_for_to_upload_to_sftp() -> bool {
    let mut input = String::new();
    println!(
        "{} {}",
        format!("Do you want to upload the backup to SFTP?").yellow(),
        format!("(y/n)").green()
    );
    std::io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    if input.eq_ignore_ascii_case("y") || input.eq_ignore_ascii_case("yes") {
        return false;
    } else if input.eq_ignore_ascii_case("n") || input.eq_ignore_ascii_case("no") {
        return true;
    } else {
        return ask_for_input_for_to_upload_to_sftp();
    }
}

fn self_update() -> bool {
    println!("Current Version: {}", cargo_crate_version!());

    // Determine the target architecture (x86_64 or aarch64)
    let target = if std::env::consts::ARCH == "x86_64" {
        "limonium-x86_64-unknown-linux-gnu.zip"
    } else if std::env::consts::ARCH == "aarch64" {
        "limonium-aarch64-unknown-linux-gnu.zip"
    } else {
        panic!("Unsupported architecture: {}", env::consts::ARCH);
    };

    println!("Target: {}", target);

    let status = self_update::backends::github::Update::configure()
        .repo_owner("andrew121410")
        .repo_name("limonium")
        .target(target)
        .bin_name("limonium")
        .no_confirm(true)
        .show_download_progress(false)
        .show_output(false)
        .current_version(cargo_crate_version!())
        .build()
        .expect("Failed to build update")
        .update()
        .expect("Failed to update");

    if status.updated() {
        println!(
            "Updated Limonium from {} to {}",
            cargo_crate_version!(),
            &status.version()
        );
        true
    } else {
        println!("Limonium is already up to date!");
        false
    }
}
