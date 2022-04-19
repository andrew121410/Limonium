use std::process::exit;
use crate::api::platform;

use async_trait::async_trait;
use std::string::String;

// https://github.com/pufferfish-gg/Pufferfish
pub struct PufferfishAPI;
#[async_trait]
impl platform::IPlatform for PufferfishAPI {

    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = PufferfishAPI::get_jar_name(&self, &project, &version, &build);
        let real_version = get_real_version(&version);

        if real_version.is_none() {
            println!("PufferfishAPI.rs -> get_download_link -> real_version is none");
            exit(1)
        }

        // Example https://ci.pufferfish.host/job/Pufferfish-1.18/63/artifact/build/libs/pufferfish-paperclip-1.18.2-R0.1-SNAPSHOT-reobf.jar
        let mut to_return = String::from("https://ci.pufferfish.host/job/Pufferfish-");
        to_return.push_str(&real_version.unwrap());
        to_return.push_str("/");
        to_return.push_str(&build);
        to_return.push_str("/artifact/build/libs/");
        to_return.push_str(&jar_name);
        return to_return;
    }

    fn get_jar_name(&self, _project: &String, version: &String, _build: &String) -> String {
        let mut to_return = String::from("pufferfish-paperclip-");
        to_return.push_str(&version);
        to_return.push_str("-R0.1-SNAPSHOT-reobf.jar");
        return to_return;
    }

    async fn is_error(&self, _project: &String, version: &String, build: &String) -> Option<String> {
        let real_version = get_real_version(&version);

        let mut link = String::from("https://ci.pufferfish.host/job/Pufferfish-");
        link.push_str(&real_version.unwrap());
        link.push_str("/");
        link.push_str(&build);
        link.push_str("/");

        let x = reqwest::get(&link).await;

        if x.is_err() {
            Some(String::from("Website has an error"));
        }

        let text_result = x.unwrap().text().await;

        if text_result.is_err() {
            return Some(String::from("is_error -> text_result"));
        }

        if text_result.unwrap().contains("404") {
            return Some(String::from("404 error"));
        }

        return None
    }

    async fn get_latest_build(&self, _project: &String, _version: &String) -> Option<String> {
        return Some(String::from("lastSuccessfulBuild"));
    }
}

pub fn get_real_version(version: &String) -> Option<String> {
    if version.contains("1.18") {
        return Some(String::from("1.18"));
    } else if version.contains("1.17") {
        return Some(String::from("1.17"));
    }
    return None;
}