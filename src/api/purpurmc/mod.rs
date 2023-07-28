use std::string::String;

use async_trait::async_trait;

use crate::api::platform;
use crate::hash_utils::Hash;
use crate::objects::DownloadedJar::DownloadedJar;

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

    async fn get_hash_from_web(&self, _project: &String, version: &String, build: &String, downloaded_jar: Option<&DownloadedJar>) -> Option<Hash> {
        let mut link = String::from("https://api.purpurmc.org/v2/purpur/");
        link.push_str(&version);
        link.push_str("/");
        link.push_str(&build);

        let text = reqwest::get(&link).await.unwrap().text().await.unwrap();
        let purpur_build_info_json: PurpurBuildInfo = serde_json::from_str(text.as_str()).unwrap();

        if purpur_build_info_json.md5.is_some() {
            return Some(Hash::new(String::from("md5"), purpur_build_info_json.md5.unwrap()));
        }
        return None;
    }

    async fn get_latest_version(&self, _project: &String) -> Option<String> {
        None
    }

    async fn custom_download_functionality(&self, _project: &String, _version: &String, _build: &String, _link: &String) -> Option<DownloadedJar> {
        None
    }
}

// Example https://api.purpurmc.org/v2/purpur/1.18.2/latest
#[derive(Deserialize, Default)]
struct PurpurBuildInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    md5: Option<String>,
}