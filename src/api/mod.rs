use std::fs::File;
use std::io;
use std::io::Cursor;
use crate::api::platform::IPlatform;

pub mod platform;
pub mod papermc;
pub mod purpurmc;
pub mod pufferfish;

pub fn get_platform(the_project: &String) -> &dyn IPlatform {
    return match the_project.as_str() {
        "pufferfish" => &pufferfish::PufferfishAPI as &dyn platform::IPlatform,
        "purpur" => &purpurmc::PurpurAPI as &dyn platform::IPlatform,
        _ => &papermc::PaperAPI as &dyn platform::IPlatform,
    }
}

pub async fn download(link: &String, path: &String){
    let response = reqwest::get(link).await.unwrap();
    let mut file = File::create(&path).unwrap();
    let mut content =  Cursor::new(response.bytes().await.unwrap());
    io::copy(&mut content, &mut file).unwrap();
}