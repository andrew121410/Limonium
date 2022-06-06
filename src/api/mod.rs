use std::env::temp_dir;
use std::fs::File;
use std::{fs, io};
use std::io::Cursor;

use crate::api::platform::IPlatform;

pub mod platform;
pub mod papermc;
pub mod purpurmc;
pub mod pufferfish;
pub mod patina;
pub mod mirai;
pub mod spigotmc;

pub fn get_platform(the_project: &String) -> &dyn IPlatform {
    return match the_project.to_lowercase().as_str() {
        "purpur" => &purpurmc::PurpurAPI as &dyn IPlatform,
        "pufferfish" => &pufferfish::PufferfishAPI as &dyn IPlatform,
        "patina" => &patina::PatinaAPI as &dyn IPlatform,
        "mirai" => &mirai::MiraiAPI as &dyn IPlatform,
        _ => &papermc::PaperAPI as &dyn IPlatform,
    };
}

// pub async fn download(link: &String, path: &String) {
//     let response = reqwest::get(link).await.unwrap();
//     let mut file = File::create(&path).unwrap();
//     let mut content = Cursor::new(response.bytes().await.unwrap());
//     io::copy(&mut content, &mut file).unwrap();
// }

pub async fn download_jar_to_temp_dir(link: &String) {
    let response = reqwest::get(link).await.unwrap();
    let path =  temp_dir().join("theServer.jar");
    let mut file = File::create(path).unwrap();
    let mut content = Cursor::new(response.bytes().await.unwrap());
    io::copy(&mut content, &mut file).unwrap();
}

pub fn copy_jar_from_temp_dir_to_dest(final_path: &String) {
    fs::copy(temp_dir().join("theServer.jar"), &final_path).expect("Failed copying jar from temp directory to final path");
}