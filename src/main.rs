#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

extern crate core;

mod api;

use futures::executor::block_on;
use std::{env, process};
use std::borrow::Borrow;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use std::string::String;
use crate::api::download;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Bruh are you serious?");
        process::exit(0x0100);
    }

    let project = args[1].to_lowercase();
    let version = args[2].to_string();
    let mut build = args[3].to_string();

    let other_args = Vec::from_iter(&args[4..args.len()]);

    if other_args.len() % 2 != 0 {
        println!("Something went wrong with otherArgs");
        process::exit(0x0100);
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

    api::download(&platform.get_download_link(&project, &version, &build), &path).await;
    println!("Downloaded {}", &path);
}