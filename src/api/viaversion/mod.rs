use std::{env, fs};
use std::env::temp_dir;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

use async_trait::async_trait;
use colored::Colorize;
use regex::Regex;

use crate::api;
use crate::api::platform;
use crate::hashutils::Hash;

// https://github.com/ViaVersion
// https://ci.viaversion.com/
pub struct ViaVersionAPI {}

#[async_trait]
impl platform::IPlatform for ViaVersionAPI {
    async fn get_latest_version(&self, project: &String) -> Option<String> {
        Some("".to_string())
    }

    async fn get_latest_build(&self, project: &String, version: &String) -> Option<String> {
        Some("".to_string())
    }

    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let mut fallback_channel = "".to_string();
        if project.eq_ignore_ascii_case("viaversion") {
            fallback_channel = "viaversion".to_string();
        } else if project.eq_ignore_ascii_case("viabackwards") {
            fallback_channel = "viabackwards".to_string();
        }

        let channel_selected = api::get_channel_or_fallback(&fallback_channel);

        // Check if the channel is valid
        if !is_valid_channel(&channel_selected) {
            println!("{} {}", "Error:".red(), "Invalid channel selected");
            return "".to_string();
        }

        // Get the download link for the .zip file
        let mut link = get_zip_download_link(&project, &channel_selected);

        return link;
    }

    fn get_jar_name(&self, project: &String, version: &String, build: &String) -> String {
        if project.eq_ignore_ascii_case("ViaVersion") {
            return "ViaVersion.jar".to_string();
        } else if project.eq_ignore_ascii_case("ViaBackwards") {
            return "ViaBackwards.jar".to_string();
        }

        println!("{} {}", "Error:".red(), "get_jar_name() called with invalid project");
        exit(1);
    }

    async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<Hash> {
        None
    }

    async fn custom_download_functionality(&self, project: &String, version: &String, build: &String, link: &String) -> Option<String> {
        // Download the .zip file to the temp directory
        let zip_file_path = env::temp_dir().join("limonium-via.zip");
        let response = reqwest::get(link).await.expect("Failed to send request.");
        let bytes = response.bytes().await.expect("Failed to get bytes.");
        fs::write(&zip_file_path, &bytes).expect("Failed to write file.");

        // Create a folder in the temp directory named "limonium-via"
        let created_folder = env::temp_dir().join("limonium-via");
        if !created_folder.exists() {
            fs::create_dir(&created_folder).unwrap();
        }

        // Move the .zip file to the "limonium-via" folder
        let new_zip_path = created_folder.join("limonium-via.zip");
        fs::rename(&zip_file_path, &new_zip_path).unwrap();

        // Extract the .zip file to the "limonium-via" folder
        let output = Command::new("unzip")
            .arg(&new_zip_path)
            .current_dir(&created_folder)
            .output()
            .expect("Failed to execute command.");

        if !output.status.success() {
            println!("Extraction failed: {:?}", output);
            return None;
        }

        let jar_pattern = Regex::new(r"^Via(Backwards|Version)-\d+\.\d+\.\d+(-SNAPSHOT)?\.jar$").unwrap();

        // Find the .jar files in the libs folder in the "limonium-via" folder
        let jar_files = fs::read_dir(&created_folder.join("libs"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                let path = entry.path();
                path.is_file() && path.extension().map_or(false, |ext| ext == "jar") && jar_pattern.is_match(path.file_name().unwrap().to_str().unwrap())
            });

        let mut the_jar_file_path: Option<PathBuf> = None;

        // Find the jar file (should only be one)
        for jar_file in jar_files {
            the_jar_file_path = Option::from(jar_file.path());
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

        // Delete the "limonium-via" folder
        fs::remove_dir_all(&created_folder).unwrap();

        // Return the name of the jar file
        let final_jar_file_name = final_jar_path.file_name().unwrap().to_str().unwrap().to_string();
        Some(final_jar_file_name)
    }
}

fn get_zip_download_link(project: &String, channel: &String) -> String {
    if project.eq_ignore_ascii_case("viaversion") && channel.eq_ignore_ascii_case("viaversion") {
        return "https://ci.viaversion.com/job/ViaVersion/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viaversion") && channel.eq_ignore_ascii_case("dev") {
        return "https://ci.viaversion.com/job/ViaVersion-DEV/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") && channel.eq_ignore_ascii_case("viabackwards") {
        return "https://ci.viaversion.com/view/ViaBackwards/job/ViaBackwards/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") && channel.eq_ignore_ascii_case("dev") {
        return "https://ci.viaversion.com/view/ViaBackwards/job/ViaBackwards-DEV/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    }

    return "na".to_string();
}

fn is_valid_channel(channel: &String) -> bool {
    return match channel.to_lowercase().as_str() {
        "viaversion" => true,
        "dev" => true,
        "viabackwards" => true,
        _ => false,
    };
}

fn fallback_channel(project: &String) -> String {
    if project.eq_ignore_ascii_case("viaversion") {
        return "viaversion".to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") {
        return "viabackwards".to_string();
    }

    return "".to_string();
}