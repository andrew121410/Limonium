use std::env::temp_dir;
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};

use colored::Colorize;
use futures_util::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::{header, Client};
use uuid::Uuid;

use crate::download_controllers::platform::IPlatform;
use crate::objects::DownloadedJar::DownloadedJar;

pub mod geysermc;
pub mod papermc;
pub mod platform;
pub mod pufferfish;
pub mod purpurmc;
pub mod spigotmc;
mod viaversion;

pub fn get_platform(the_project: &String) -> &dyn IPlatform {
    return match the_project.to_lowercase().as_str() {
        "purpur" => &purpurmc::PurpurAPI as &dyn IPlatform,
        "pufferfish" => &pufferfish::PufferfishAPI as &dyn IPlatform,
        "geyser" | "floodgate" => &geysermc::GeyserAPI {} as &dyn IPlatform,
        "viaversion" | "viabackwards" => &viaversion::ViaVersionAPI {} as &dyn IPlatform,
        "bungeecord" => &spigotmc::bungeecord::BungeeCordAPI {} as &dyn IPlatform,
        _ => &papermc::PaperAPI {} as &dyn IPlatform,
    };
}

pub fn is_valid_platform(the_project: &String) -> bool {
    return match the_project.to_lowercase().as_str() {
        "spigot" => true, // A message will be displayed to the user saying that Spigot must be compiled.
        "bungeecord" => true,

        "purpur" => true,
        "pufferfish" => true,

        "paper" => true,
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
    headers.insert(header::USER_AGENT, "rust-reqwest/limonium".parse().unwrap());

    // This seems to break some downloads?
    // headers.insert(
    //     header::ACCEPT,
    //     "application/octet-stream".parse().unwrap(),
    // );

    let response = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .unwrap()
        .get(link)
        .headers(headers)
        .send()
        .await
        .unwrap();

    // If the response is not successful, we should alert not exit though
    if !response.status().is_success() {
        println!("{} {}", "Failed to download file from".red(), link);
        println!("{} {}", "Status code:".red(), response.status());
        println!(
            "{} {}",
            "Status text:".red(),
            response.status().canonical_reason().unwrap()
        );
    }

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

pub async fn download_jar_to_temp_dir_with_progress_bar(link: &String) -> DownloadedJar {
    let tmp_jar_name = random_file_name(&".jar".to_string());

    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, "rust-reqwest/limonium".parse().unwrap());

    // This seems to break some downloads?
    // headers.insert(
    //     header::ACCEPT,
    //     "application/octet-stream".parse().unwrap(),
    // );

    // Create a new HTTP client
    let client = Client::new();

    // Get the content length of the file
    let content_length = client
        .head(link)
        .send()
        .await
        .expect("Failed to get content length")
        .content_length()
        .expect("No content length header");

    println!("{}", format!("{}", "Downloading...").bright_green());

    let pb = ProgressBar::new(content_length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {bar} {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Create a new file to write the downloaded data
    let path = temp_dir().join(&tmp_jar_name);
    let mut file = File::create(path).unwrap();

    // Create a stream of the file data
    let mut stream = client
        .get(link)
        .send()
        .await
        .expect("Failed to get file data?")
        .bytes_stream();

    // Loop over the stream and write to the file
    while let Some(item) = stream.next().await {
        // Get the chunk of data
        let chunk = item.expect("Failed to get chunk");

        // Write the chunk to the file
        file.write_all(&chunk)
            .expect("Failed to write_all of chunk?");

        // Update the progress bar
        pb.inc(chunk.len() as u64);
    }

    // Finish the progress bar
    pb.finish_and_clear();

    return DownloadedJar {
        real_jar_name: None, // We might not know the real jar name
        temp_jar_name: tmp_jar_name.clone(),
        temp_jar_path: temp_dir().join(&tmp_jar_name),
    };
}

pub fn copy_jar_from_temp_dir_to_dest(tmp_jar_name: &String, final_path: &String) {
    fs::copy(temp_dir().join(&tmp_jar_name), &final_path)
        .expect("Failed copying jar from temp directory to final path");
}

pub(crate) fn find_jar_files(dir: &Path, jar_pattern: &Regex) -> Vec<PathBuf> {
    let mut jar_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "jar"
                            && jar_pattern.is_match(path.file_name().unwrap().to_str().unwrap())
                        {
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
