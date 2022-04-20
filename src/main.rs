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

mod api;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("{} {} {}", format!("Something went wrong!").red().bold(), format!("Example:").yellow(), format!("./limonium paper 1.18.2 latest").green());
        process::exit(101);
    }

    let project = args[1].to_lowercase();
    let version = args[2].to_string();
    let mut build = args[3].to_string();

    let other_args = Vec::from_iter(&args[4..args.len()]);

    let mut other_args_map: HashMap<String, String> = HashMap::new();
    let mut i = 0;
    while i < other_args.len() {
        let current = other_args[i];

        match current.to_lowercase().as_str() {
            "--o" | "--n" => {
                other_args_map.insert(current.clone(), other_args[i + 1].clone());
                i += 2;
            }
            _ => {
                other_args_map.insert(current.clone(), String::from(""));
                i += 1;
            }
        }
    }

    let mut path: String = String::from("");
    if other_args_map.contains_key(&String::from("--o")) {
        path.push_str(&other_args_map[&String::from("--o")]);
    }

    if other_args_map.contains_key(&String::from("--n")) {
        path.push_str(&other_args_map[&String::from("--n")]);
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

        if build.eq_ignore_ascii_case("latest") {
            build = platform.get_latest_build(&project, &version).await.expect("Getting the latest build failed?");
        }

        if path.eq("") {
            path.push_str(platform.get_jar_name(&project, &version, &build).as_str());
        }

        let start = Instant::now();

        // Check for any problems before trying to download the .jar
        let is_error = platform.is_error(&project, &version, &build).await;
        if is_error.is_some() {
            println!("{} {}", format!("Platform is_error returned:").red().bold(), format!("{}", is_error.unwrap()).yellow().bold());
            process::exit(101);
        }

        api::download(&platform.get_download_link(&project, &version, &build), &path).await;

        let duration = start.elapsed().as_millis().to_string();

        println!("{} {} {} {}", format!("Downloaded:").green().bold(), format!("{}", &path.as_str()).blue().bold(), format!("Time In Milliseconds:").purple().bold(), format!("{}", &duration).yellow().bold());
    }

    if other_args_map.contains_key(&String::from("--self-update")) {
        tokio::task::spawn_blocking(move || {
            if let Err(e) = update() {
                println!("[ERROR] {}", e);
                ::std::process::exit(1);
            }
        }).await.expect("Something went wrong at self update?");
    }
}

fn update() -> Result<(), Box<dyn ::std::error::Error>> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("andrew121410")
        .repo_name("Limonium")
        .bin_name("limonium")
        .target("limonium")
        .no_confirm(true)
        .current_version(cargo_crate_version!())
        .show_download_progress(true)
        .build()?
        .update()?;
    println!("Update status: `{}`!", status.version());
    Ok(())
}