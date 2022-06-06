use std::collections::HashMap;
use std::string::String;

use async_trait::async_trait;

use crate::api::platform;

// https://github.com/PurpurMC/Purpur
pub struct PurpurAPI;

#[async_trait]
impl platform::IPlatform for PurpurAPI {
    fn get_download_link(&self, _project: &String, version: &String, build: &String) -> String {
        let mut to_return = String::from("https://api.purpurmc.org/v2/purpur/");
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

    async fn get_latest_build(&self, _project: &String, _version: &String) -> Option<String> {
        // Thank you purpur for keeping the latest tag <3
        return Some(String::from("latest"));
    }

    async fn get_jar_hash(&self, _project: &String, version: &String, build: &String) -> Option<HashMap<String, String>> {
        let mut link = String::from("https://api.purpurmc.org/v2/purpur/");
        link.push_str(&version);
        link.push_str("/");
        link.push_str(&build);

        let text = reqwest::get(&link).await.unwrap().text().await.unwrap();
        let purpur_build_info_json: PurpurBuildInfo = serde_json::from_str(text.as_str()).unwrap();

        if purpur_build_info_json.md5.is_some() {
            let mut hashmap: HashMap<String, String> = HashMap::new();
            hashmap.insert(String::from("algorithm"), String::from("md5"));
            hashmap.insert(String::from("hash"), purpur_build_info_json.md5.unwrap());
            return Some(hashmap);
        }
        return None;
    }
}

// Example https://api.purpurmc.org/v2/purpur/1.18.2/latest
#[derive(Deserialize, Default)]
struct PurpurBuildInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    md5: Option<String>,
}