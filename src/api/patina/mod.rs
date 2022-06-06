use std::collections::HashMap;
use std::string::String;

use async_trait::async_trait;

use crate::api::platform;

// https://github.com/PatinaMC/Patina
pub struct PatinaAPI;
#[async_trait]
impl platform::IPlatform for PatinaAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = PatinaAPI::get_jar_name(&self, &project, &version, &build);

        // Example https://github.com/PatinaMC/Patina/raw/releases/1.18.2/patina-paperclip-1.18.2-R0.1-SNAPSHOT-reobf.jar
        let mut link = String::from("https://github.com/PatinaMC/Patina/raw/releases/");
        link.push_str(&version);
        link.push_str("/");
        link.push_str(&jar_name);
        return link;
    }

    fn get_jar_name(&self, _project: &String, version: &String, _build: &String) -> String {
        let mut jar_name = String::from("patina-paperclip-");
        jar_name.push_str(&version);
        jar_name.push_str("-R0.1-SNAPSHOT-reobf.jar");
        return jar_name;
    }

    async fn get_latest_build(&self, _project: &String, _version: &String) -> Option<String> {
        return Some(String::from("Not needed"));
    }

    async fn get_jar_hash(&self, _project: &String, _version: &String, _build: &String) -> Option<HashMap<String, String>> {
        return None;
    }
}