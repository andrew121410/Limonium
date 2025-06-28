use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use colored::Colorize;
use futures_util::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header, Client};

use crate::download_controllers::platform::IPlatform;
use crate::file_utils;
pub(crate) use crate::objects::downloaded_file::DownloadedFile;

pub mod geysermc;
pub mod papermc;
pub mod platform;
pub mod pufferfish;
pub mod purpurmc;
pub mod spigotmc;
mod viaversion;
mod citizens;

pub fn get_platform(the_project: &String) -> &dyn IPlatform {
    match the_project.to_lowercase().as_str() {
        "purpur" => &purpurmc::PurpurAPI as &dyn IPlatform,
        "pufferfish" => &pufferfish::PufferfishAPI as &dyn IPlatform,
        "geyser" | "floodgate" => &geysermc::GeyserAPI {} as &dyn IPlatform,
        "viaversion" | "viabackwards" => &viaversion::ViaVersionAPI {} as &dyn IPlatform,
        "bungeecord" => &spigotmc::bungeecord::BungeeCordAPI {} as &dyn IPlatform,
        "citizens" | "citizens2" => &citizens::Citizens2API {} as &dyn IPlatform,
        _ => &papermc::PaperAPI {} as &dyn IPlatform,
    }
}

pub fn is_valid_platform(the_project: &String) -> bool {
    match the_project.to_lowercase().as_str() {
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

        "citizens" | "citizens2" => true,
        _ => false,
    }
}

pub async fn download_file_to_temp_dir_with_progress_bar(link: &String, extension: &String, temp_directory: &PathBuf) -> DownloadedFile {
    let tmp_file_name = file_utils::random_file_name(&extension);

    println!("{}", format!("{}", "Downloading...").bright_green());

    let client = Client::new();
    let response = client
        .get(link)
        .headers(limonium_headers())
        .send()
        .await
        .expect("Failed to get file data?");

    let content_length = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(content_length);
    pb.set_style(
        ProgressStyle::with_template(
            "⬇️  {msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes:>7}/{total_bytes:7} ({bytes_per_sec}, {eta})",
        )
            .unwrap()
            .progress_chars("█░-"),
    );

    let path = temp_directory.join(&tmp_file_name);
    let mut file = File::create(&path).unwrap();

    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.expect("Failed to get chunk");
        file.write_all(&chunk).expect("Failed to write_all of chunk?");
        pb.inc(chunk.len() as u64);
    }

    pb.finish_and_clear();

    DownloadedFile {
        real_file_name: None,
        temp_file_name: tmp_file_name.clone(),
        temp_file_path: path,
    }
}

fn limonium_headers() -> reqwest::header::HeaderMap {
    let version = env!("CARGO_PKG_VERSION");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        format!("Limonium/{} (https://github.com/andrew121410/Limonium)", version)
            .parse()
            .unwrap(),
    );
    headers
}