extern crate core;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::{env, process};
use std::collections::HashMap;
use std::string::String;
use std::time::Instant;

use colored::Colorize;

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

    if other_args.len() % 2 != 0 {
        println!("other_args make sure it's divisible by 2!");
        process::exit(101);
    }

    let mut other_args_map = HashMap::new();
    let mut i = 0;
    while i < other_args.len() {
        other_args_map.insert(other_args[i], other_args[i + 1]);
        i += 2;
    }

    let platform = api::get_platform(&project);

    if build.eq_ignore_ascii_case("latest") {
        build = platform.get_latest_build(&project, &version).await.expect("MAIN get latest build returned none");
    }

    let mut path: String = String::from("");
    if other_args_map.contains_key(&String::from("--o")) {
        path.push_str(other_args_map[&String::from("--o")]);
    }

    if other_args_map.contains_key(&String::from("--n")) {
        path.push_str(other_args_map[&String::from("--n")]);
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