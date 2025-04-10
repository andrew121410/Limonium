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
use crate::file_utils;
pub(crate) use crate::objects::DownloadedJar::DownloadedJar;

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

pub async fn download_jar_to_temp_dir(link: &String) -> DownloadedJar {
    let tmp_jar_name = file_utils::random_file_name(&".jar".to_string());

    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, "rust-reqwest/limonium".parse().unwrap());

    // This seems to break some downloads?
    // headers.insert(
    //     header::ACCEPT,
    //     "application/octet-stream".parse().unwrap(),
    // );

    let response = Client::builder()
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

    let path = file_utils::get_or_create_limonium_dir().join(&tmp_jar_name);
    let mut file = File::create(path).unwrap();
    let mut content = Cursor::new(response.bytes().await.unwrap());
    io::copy(&mut content, &mut file).unwrap();

    DownloadedJar {
        real_jar_name: None, // We might not know the real jar name
        temp_jar_name: tmp_jar_name.clone(),
        temp_jar_path: file_utils::get_or_create_limonium_dir().join(&tmp_jar_name),
    }
}

pub async fn download_jar_to_temp_dir_with_progress_bar(link: &String) -> DownloadedJar {
    let tmp_jar_name = file_utils::random_file_name(&".jar".to_string());

    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, "rust-reqwest/limonium".parse().unwrap());

    println!("{}", format!("{}", "Downloading...").bright_green());

    let client = Client::new();
    let response = client
        .get(link)
        .headers(headers.clone())
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

    let path = file_utils::get_or_create_limonium_dir().join(&tmp_jar_name);
    let mut file = File::create(&path).unwrap();

    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.expect("Failed to get chunk");
        file.write_all(&chunk).expect("Failed to write_all of chunk?");
        pb.inc(chunk.len() as u64);
    }

    pb.finish_and_clear();

    DownloadedJar {
        real_jar_name: None,
        temp_jar_name: tmp_jar_name.clone(),
        temp_jar_path: path,
    }
}