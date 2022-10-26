use std::collections::HashMap;
use std::string::String;

use async_trait::async_trait;

use crate::api::platform;
use crate::hashutils::Hash;

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

    async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<Hash> {
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

            return Some(Hash::new(String::from("sha256"), sha256.clone()));
        }
        return None;
    }
}

// Example https://api.papermc.io/v2/projects/paper/versions/1.19.2
#[derive(Deserialize, Default)]
struct PaperBuildsJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    builds: Option<Vec<i64>>,
}

// Example https://api.papermc.io/v2/projects/paper/versions/1.19.2/builds/237
#[derive(Deserialize, Default)]
struct PaperBuildInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    downloads: Option<HashMap<String, HashMap<String, String>>>,
}