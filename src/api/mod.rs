use std::{env, fs, io};
use std::env::temp_dir;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::ArgMatches;
use colored::Colorize;
use regex::Regex;
use reqwest::header;
use uuid::Uuid;

use crate::{api, SUB_COMMAND_ARG_MATCHES};
use crate::api::platform::IPlatform;

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
        "geyser" => &geysermc::GeyserAPI {} as &dyn IPlatform,
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

pub async fn download_jar_to_temp_dir(link: &String) -> String {
    let mut tmp_jar_name = random_file_name(&".jar".to_string());

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

    return tmp_jar_name;
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

// Returns file name found in the /tmp directory
async fn jenkins_artifacts_bundle_zip_download_and_find_jar_and_place_jar_in_the_tmp_directory(project: &String, version: &String, build: &String, link: &String, regex: &str) -> Option<String> {
    let random_zip_name = random_file_name(&".zip".to_string());
    let random_folder_name = random_file_name(&"".to_string());

    // Download the .zip file to the temp directory
    let zip_file_path = env::temp_dir().join(&random_zip_name);
    let response = reqwest::get(link).await.expect("Failed to send request.");
    let bytes = response.bytes().await.expect("Failed to get bytes.");
    fs::write(&zip_file_path, &bytes).expect("Failed to write file.");

    // Create a folder in the temp directory with a random name
    let created_folder = env::temp_dir().join(&random_folder_name);
    if !created_folder.exists() {
        fs::create_dir(&created_folder).unwrap();
    }

    // Move the .zip file to the created folder
    let new_zip_path = created_folder.join(&random_zip_name);
    fs::rename(&zip_file_path, &new_zip_path).unwrap();

    // Extract the .zip file in the created folder
    let output = Command::new("unzip")
        .arg(&new_zip_path)
        .current_dir(&created_folder)
        .output()
        .expect("Failed to execute command.");

    if !output.status.success() {
        println!("Extraction failed: {:?}", output);
        return None;
    }

    // Delete the .zip file
    fs::remove_file(&new_zip_path).unwrap();

    // Find the .jar files in the created folder
    let jar_pattern = Regex::new(regex).unwrap();
    let jar_files = find_jar_files(&created_folder, &jar_pattern);

    let mut the_jar_file_path: Option<PathBuf> = None;
    // Find the jar file (should only be one)
    for jar_file in jar_files {
        the_jar_file_path = Some(jar_file);
    }

    // Don't continue if the jar file was not found
    if the_jar_file_path.is_none() {
        println!("{} {}", "Error:".red(), "Failed to find jar file");
        return None;
    }

    // Generate a random name for the jar file
    let random_jar_name = api::random_file_name(&".jar".to_string());

    // Move the jar file to the temp directory
    let final_jar_path = env::temp_dir().join(&random_jar_name);
    fs::rename(&the_jar_file_path.unwrap(), &final_jar_path).unwrap();

    // Delete the created folder
    fs::remove_dir_all(&created_folder).unwrap();

    // Return the name of the jar file
    let final_jar_file_name = final_jar_path.file_name().unwrap().to_str().unwrap().to_string();
    Some(final_jar_file_name)
}

fn find_jar_files(dir: &Path, jar_pattern: &Regex) -> Vec<PathBuf> {
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