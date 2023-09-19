use std::{fs, io};
use std::env::temp_dir;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use colored::Colorize;
use regex::Regex;
use reqwest::header;
use uuid::Uuid;

use crate::SUB_COMMAND_ARG_MATCHES;
use crate::api::platform::IPlatform;
use crate::objects::DownloadedJar::DownloadedJar;

pub mod platform;
pub mod papermc;
pub mod purpurmc;
pub mod pufferfish;
pub mod spigotmc;
pub mod geysermc;
mod viaversion;

pub fn get_platform(the_project: &String) -> &dyn IPlatform {
    return match the_project.to_lowercase().as_str() {
        "purpur" => &purpurmc::PurpurAPI as &dyn IPlatform,
        "pufferfish" => &pufferfish::PufferfishAPI as &dyn IPlatform,
        "geyser" | "floodgate" => &geysermc::GeyserAPI {} as &dyn IPlatform,
        "viaversion" | "viabackwards" => &viaversion::ViaVersionAPI {} as &dyn IPlatform,
        _ => &papermc::PaperAPI {} as &dyn IPlatform,
    };
}

pub fn is_valid_platform(the_project: &String) -> bool {
    return match the_project.to_lowercase().as_str() {
        "spigot" => true,
        "purpur" => true,
        "pufferfish" => true,

        "paper" => true,
        "waterfall" => true,
        "velocity" => true,

        "geyser" => true,
        "floodgate" => true,

        "viaversion" => true,
        "viabackwards" => true,
        _ => false,
    };
}

pub fn random_file_name(fileExtension: &String) -> String {
    let mut tmp_jar_name = String::from("limonium-");
    tmp_jar_name.push_str(&Uuid::new_v4().to_string());
    tmp_jar_name.push_str(fileExtension);
    return tmp_jar_name;
}

pub async fn download_jar_to_temp_dir(link: &String) -> DownloadedJar {
    let tmp_jar_name = random_file_name(&".jar".to_string());

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        "rust-reqwest/limonium".parse().unwrap(),
    );
    headers.insert(
        header::ACCEPT,
        "application/octet-stream".parse().unwrap(),
    );
    let response = reqwest::Client::new()
        .get(link)
        .headers(headers)
        .send().await.unwrap();

    let path = temp_dir().join(&tmp_jar_name);
    let mut file = File::create(path).unwrap();
    let mut content = Cursor::new(response.bytes().await.unwrap());
    io::copy(&mut content, &mut file).unwrap();

    return DownloadedJar {
        real_jar_name: None, // We might not know the real jar name
        temp_jar_name: tmp_jar_name.clone(),
        temp_jar_path: temp_dir().join(&tmp_jar_name),
    };
}

pub fn copy_jar_from_temp_dir_to_dest(tmp_jar_name: &String, final_path: &String) {
    fs::copy(temp_dir().join(&tmp_jar_name), &final_path).expect("Failed copying jar from temp directory to final path");
}

pub fn get_channel_or_fallback(fallback: &String) -> String {
    unsafe {
        let args: &ArgMatches = SUB_COMMAND_ARG_MATCHES.as_ref().expect("SUB_COMMAND_ARG_MATCHES is not set");
        let channel = args.get_one::<String>("channel").unwrap_or(&fallback);
        return channel.to_string();
    }
}

pub(crate) fn find_jar_files(dir: &Path, jar_pattern: &Regex) -> Vec<PathBuf> {
    let mut jar_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "jar" && jar_pattern.is_match(path.file_name().unwrap().to_str().unwrap()) {
                            jar_files.push(path.clone());
                        }
                    }
                } else if path.is_dir() {
                    jar_files.extend(find_jar_files(&path, jar_pattern));
                }
            }
        }
    }

    jar_files
}