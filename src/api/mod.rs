use std::{fs, io};
use std::env::temp_dir;
use std::fs::File;
use std::io::Cursor;

use reqwest::header;
use uuid::Uuid;

use crate::api::platform::IPlatform;

pub mod platform;
pub mod papermc;
pub mod purpurmc;
pub mod pufferfish;
pub mod mirai;
pub mod spigotmc;
pub mod bloomhost;

pub fn get_platform(the_project: &String) -> &dyn IPlatform {
    return match the_project.to_lowercase().as_str() {
        "petal" => &bloomhost::PetalAPI as &dyn IPlatform,
        "purpur" => &purpurmc::PurpurAPI as &dyn IPlatform,
        "pufferfish" => &pufferfish::PufferfishAPI as &dyn IPlatform,
        "mirai" => &mirai::MiraiAPI as &dyn IPlatform,
        _ => &papermc::PaperAPI as &dyn IPlatform,
    };
}

pub fn is_valid_platform(the_project: &String) -> bool {
    return match the_project.to_lowercase().as_str() {
        "spigot" => true,
        "petal" => true,
        "purpur" => true,
        "pufferfish" => true,
        "mirai" => true,
        "paper" => true,
        "waterfall" => true,
        "velocity" => true,
        _ => false,
    };
}

// pub async fn download(link: &String, path: &String) {
//     let response = reqwest::get(link).await.unwrap();
//     let mut file = File::create(&path).unwrap();
//     let mut content = Cursor::new(response.bytes().await.unwrap());
//     io::copy(&mut content, &mut file).unwrap();
// }

pub async fn download_jar_to_temp_dir(link: &String) -> String {
    let mut tmp_jar_name = String::from("limonium-");
    tmp_jar_name.push_str(&Uuid::new_v4().to_string());
    tmp_jar_name.push_str(".jar");

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