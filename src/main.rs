extern crate core;
#[macro_use]
extern crate self_update;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::{env, process};
use std::collections::HashMap;
use std::string::String;
use std::time::Instant;

use colored::Colorize;

use crate::api::spigotmc::SpigotAPI;
use crate::hashutils::Hash;

mod api;
mod hashutils;
mod githubutils;
mod server_jars_com;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("{} {} {}", format!("Something went wrong!").red().bold(), format!("Example:").yellow(), format!("./limonium paper 1.19.2").green());
        process::exit(102);
    }

    let mut args_map: HashMap<String, String> = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        let current = args[i].clone();

        match current.to_lowercase().as_str() {
            "--o" | "--n" => {
                args_map.insert(current, args[i + 1].clone());
                i += 2;
            }
            _ => {
                args_map.insert(current, String::from(""));
                i += 1;
            }
        }
    }

    // Handle path arguments
    let mut path: String = String::from("");
    if args_map.contains_key(&String::from("--o")) {
        path.push_str(&args_map[&String::from("--o")]);
    }
    if args_map.contains_key(&String::from("--n")) {
        path.push_str(&args_map[&String::from("--n")]);
    }

    let project = args[1].to_lowercase();
    let version = args[2].to_string();

    // Handle downloading from ServerJars.com ONLY if --server-jars-com is passed
    if args_map.contains_key(&"--server-jars-com".to_string()) {
        server_jars_com::download_jar(&project, &version, &mut path).await;
        return; // Don't continue
    }

    if !api::is_valid_platform(&project) {
        println!("{} {} {} {}", format!("Something went wrong!").red().bold(), format!("Project").yellow(), format!("{}", &project).red(), format!("is not valid!").yellow());
        process::exit(102);
    }

    // Spigot is special because it's dumb
    if project.eq_ignore_ascii_case("spigot") {
        if path.eq("") {
            path.push_str("./spigot-");
            path.push_str(&version);
            path.push_str(".jar");
        }

        SpigotAPI::download_build_tools();
        SpigotAPI::run_build_tools(&version, &path);
    } else {
        let platform = api::get_platform(&project);
        let build = platform.get_latest_build(&project, &version).await.expect("Getting the latest build failed?");

        if path.eq("") {
            path.push_str(platform.get_jar_name(&project, &version, &build).as_str());
        }

        let start = Instant::now();

        let tmp_jar_name = api::download_jar_to_temp_dir(&platform.get_download_link(&project, &version, &build)).await;

        // Verify hash of jar if possible
        let hash_optional = platform.get_jar_hash(&project, &version, &build).await;
        if hash_optional.is_some() {
            hashutils::validate_the_hash(&hash_optional.unwrap(), &tmp_jar_name);
        } else {
            println!("{}", format!("Not checking hash!").yellow().bold());
        }

        api::copy_jar_from_temp_dir_to_dest(&tmp_jar_name, &path);

        let duration = start.elapsed().as_millis().to_string();
        println!("{} {} {} {}", format!("Downloaded JAR:").green().bold(), format!("{}", &path.as_str()).blue().bold(), format!("Time In Milliseconds:").purple().bold(), format!("{}", &duration).yellow().bold());
    }

    if args_map.contains_key(&String::from("--self-update")) {
        tokio::task::spawn_blocking(move || {
            if let Err(e) = update() {
                println!("[ERROR] {}", e);
                ::std::process::exit(1);
            }
        }).await.expect("Something went wrong at self update?");
    }
}

fn update() -> Result<(), Box<dyn ::std::error::Error>> {
    println!("Current Version: {}", cargo_crate_version!());
    let status = self_update::backends::github::Update::configure()
        .repo_owner("andrew121410")
        .repo_name("limonium")
        .target("limonium-linux.zip")
        .bin_name("limonium")
        .no_confirm(true)
        .show_download_progress(false)
        .show_output(false)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    if status.updated() {
        println!("Updated Limonium from {} to {}", cargo_crate_version!(), &status.version());
    } else {
        println!("Already up to date!");
    }
    Ok(())
}