use std::collections::HashMap;
use std::string::String;

use async_trait::async_trait;

use crate::api::platform;

// https://github.com/PaperMC
pub struct PaperAPI;

#[async_trait]
impl platform::IPlatform for PaperAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = PaperAPI::get_jar_name(&self, &project, &version, &build);

        let mut to_return = String::from("https://api.papermc.io/v2/projects/");
        to_return.push_str(&project);
        to_return.push_str("/versions/");
        to_return.push_str(&version);
        to_return.push_str("/builds/");
        to_return.push_str(&build);
        to_return.push_str("/downloads/");
        to_return.push_str(&jar_name);

        return to_return;
    }

    fn get_jar_name(&self, project: &String, version: &String, build: &String) -> String {
        let mut to_return = String::from("");
        to_return.push_str(&project);
        to_return.push_str("-");
        to_return.push_str(&version);
        to_return.push_str("-");
        to_return.push_str(&build);
        to_return.push_str(".jar");
        return to_return;
    }

    async fn is_error(&self, project: &String, version: &String, build: &String) -> Option<String> {
        let mut link = String::from("https://api.papermc.io/v2/projects/");
        link.push_str(&project);
        link.push_str("/versions/");
        link.push_str(&version);
        link.push_str("/builds/");
        link.push_str(&build);

        let x = reqwest::get(&link).await;

        if x.is_err() {
            Some(String::from("Website has an error"));
        }

        let text_result = x.unwrap().text().await;

        if text_result.is_err() {
            return Some(String::from("text_result.is_err()"));
        }

        let text = text_result.unwrap();

        let json_value: Result<PaperBuildsJSON, _> = serde_json::from_str(&text);

        if json_value.is_err() {
            return Some(String::from("JSON Value has some issue"));
        }

        return None;
    }

    async fn get_latest_build(&self, project: &String, version: &String) -> Option<String> {
        let mut link = String::from("https://api.papermc.io/v2/projects/");
        link.push_str(&project);
        link.push_str("/versions/");
        link.push_str(&version);

        let text = reqwest::get(&link).await.unwrap().text().await.unwrap();
        let paper_json: PaperBuildsJSON = serde_json::from_str(text.as_str()).unwrap();

        if paper_json.error.is_some() {
            return None;
        }

        if paper_json.builds.is_none() {
            return None;
        }

        return Some(paper_json.builds.unwrap().iter().max().unwrap().to_string());
    }

    async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<HashMap<String, String>> {
        let mut link = String::from("https://api.papermc.io/v2/projects/");
        link.push_str(&project);
        link.push_str("/versions/");
        link.push_str(&version);
        link.push_str("/builds/");
        link.push_str(&build);

        let text = reqwest::get(&link).await.unwrap().text().await.unwrap();
        let paper_build_info_json: PaperBuildInfo = serde_json::from_str(text.as_str()).unwrap();

        if paper_build_info_json.downloads.is_some() {
            let downloads = paper_build_info_json.downloads.unwrap();
            let sha256: &String = downloads.get("application").unwrap().get("sha256").unwrap();

            let mut hashmap: HashMap<String, String> = HashMap::new();
            hashmap.insert(String::from("algorithm"), String::from("sha256"));
            hashmap.insert(String::from("hash"), sha256.clone());

            return Some(hashmap);
        }
        return None;
    }
}

// Example https://api.papermc.io/v2/projects/paper/versions/1.18.2
#[derive(Deserialize, Default)]
struct PaperBuildsJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    builds: Option<Vec<i64>>,
}

// Example https://api.papermc.io/v2/projects/paper/versions/1.18.2/builds/375
#[derive(Deserialize, Default)]
struct PaperBuildInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    downloads: Option<HashMap<String, HashMap<String, String>>>,
}