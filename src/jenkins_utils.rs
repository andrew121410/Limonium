use std::{env, fs};
use std::path::PathBuf;
use std::process::Command;

use colored::Colorize;
use regex::Regex;

use crate::api;
use crate::hash_utils::Hash;
use crate::objects::DownloadedJar::DownloadedJar;

// Returns hash of the file fingerprint found on the Jenkins page (md5)
pub async fn extract_file_fingerprint_hash(url: &String) -> Hash {
    // Get the HTML
    let response = reqwest::get(url).await;
    let html = response.unwrap().text().await.unwrap();

    // Extract the MD5 hash using regex
    let re = Regex::new(r#"The fingerprint (\w{32})"#).unwrap();
    let captures = re.captures(&html).expect("Failed to extract MD5 hash");
    let md5_hash = captures.get(1).unwrap().as_str();

    let hash = Hash {
        algorithm: String::from("md5"),
        hash: String::from(md5_hash),
    };

    return hash;
}

// Returns file name found in the /tmp directory
pub async fn jenkins_artifacts_bundle_zip_download_and_find_jar_and_place_jar_in_the_tmp_directory(_project: &String, _version: &String, _build: &String, link: &String, regex: &str) -> Option<DownloadedJar> {
    let random_zip_name = api::random_file_name(&".zip".to_string());
    let random_folder_name = api::random_file_name(&"".to_string());

    // Create a folder in the temp directory with a random name
    let created_folder = env::temp_dir().join(&random_folder_name);
    if !created_folder.exists() {
        fs::create_dir(&created_folder).unwrap();
    }

    // Download the .zip file to the created folder
    let zip_file_path = created_folder.join(&random_zip_name);
    let response = reqwest::get(link).await.expect("Failed to send request.");
    let bytes = response.bytes().await.expect("Failed to get bytes.");
    fs::write(&zip_file_path, &bytes).expect("Failed to write file.");

    // Extract the .zip file in the created folder
    let output = Command::new("unzip")
        .arg(&zip_file_path)
        .current_dir(&created_folder)
        .output()
        .expect("Failed to execute command.");

    if !output.status.success() {
        println!("Extraction failed: {:?}", output);
        return None;
    }

    // Delete the .zip file in the created folder
    fs::remove_file(&zip_file_path).unwrap();

    // Find the .jar using the regex
    let jar_pattern = Regex::new(regex).unwrap();
    let jar_files = api::find_jar_files(&created_folder, &jar_pattern);

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

    // Get the name of the jar file
    let jar_file_name = the_jar_file_path.clone().unwrap().file_name().unwrap().to_str().unwrap().to_string();

    // Generate a random name for the jar file
    let random_jar_name = api::random_file_name(&".jar".to_string());

    // Move the jar file to the temp directory with the random name
    let final_jar_path = env::temp_dir().join(&random_jar_name);
    fs::rename(&the_jar_file_path.unwrap(), &final_jar_path).unwrap();

    // Delete the created folder
    fs::remove_dir_all(&created_folder).unwrap();

    // Name of the jar file in the temp directory (random name)
    let final_jar_file_name = final_jar_path.file_name().unwrap().to_str().unwrap().to_string();

    let downloaded_jar: DownloadedJar = DownloadedJar {
        real_jar_name: Some(jar_file_name),
        temp_jar_name: final_jar_file_name,
        temp_jar_path: final_jar_path,
    };

    return Some(downloaded_jar);
}