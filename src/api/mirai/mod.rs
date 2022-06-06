use std::collections::HashMap;
use std::string::String;

use async_trait::async_trait;

use crate::api::platform;

// https://github.com/etil2jz/Mirai
pub struct MiraiAPI;
#[async_trait]
impl platform::IPlatform for MiraiAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = MiraiAPI::get_jar_name(&self, &project, &version, &build);

        // Example https://github.com/etil2jz/Mirai/releases/download/1.18.2/mirai-paperclip-1.18.2-R0.1-SNAPSHOT-reobf.jar
        let mut link = String::from("https://github.com/etil2jz/Mirai/releases/download/");
        link.push_str(&version);
        link.push_str("/");
        link.push_str(&jar_name);
        return link;
    }

    fn get_jar_name(&self, _project: &String, version: &String, _build: &String) -> String {
        let mut jar_name = String::from("mirai-paperclip-");
        jar_name.push_str(&version);
        jar_name.push_str("-R0.1-SNAPSHOT-reobf.jar");
        return jar_name;
    }

    async fn is_error(&self, _project: &String, _version: &String, _build: &String) -> Option<String> {
        // We can't check for an error like a 404 because Mirai use Github Releases
        return None;
    }

    async fn get_latest_build(&self, _project: &String, _version: &String) -> Option<String> {
        return Some(String::from("Not needed"));
    }

    async fn get_jar_hash(&self, _project: &String, _version: &String, _build: &String) -> Option<HashMap<String, String>> {
        return None;
    }
}