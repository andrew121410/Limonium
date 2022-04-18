use crate::api::platform;

use async_trait::async_trait;
use std::string::String;

pub struct PurpurAPI;
#[async_trait]
impl platform::IPlatform for PurpurAPI {

    fn get_download_link(&self, _project: &String, version: &String, build: &String) -> String {
        let mut to_return = String::from("https://api.pl3x.net/v2/purpur/");
        to_return.push_str(&version);
        to_return.push_str("/");
        to_return.push_str(&build);
        to_return.push_str("/download");
        return to_return;
    }

    fn get_jar_name(&self, _project: &String, version: &String, build: &String) -> String {
        let mut to_return = String::from("purpur-");
        to_return.push_str(&version);
        to_return.push_str("-");
        to_return.push_str(&build);
        to_return.push_str(".jar");
        return to_return;
    }

    async fn is_error(&self, _project: &String, version: &String, build: &String) -> Option<String> {
        let mut link = String::from("https://api.pl3x.net/v2/purpur/");
        link.push_str(&version);
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
        // Thank you purpur for keeping the latest tag <3
        return Some(String::from("latest"));
    }
}