use crate::api::platform;

use async_trait::async_trait;
use std::string::String;

// https://github.com/PaperMC
pub struct PaperAPI;
#[async_trait]
impl platform::IPlatform for PaperAPI {

    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = PaperAPI::get_jar_name(&self, &project, &version, &build);

        let mut to_return = String::from("https://papermc.io/api/v2/projects/");
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
        let mut link = String::from("https://papermc.io/api/v2/projects/");
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

        let json_value: Result<PaperJSON, _> = serde_json::from_str(&text);

        if json_value.is_err() {
            return Some(String::from("JSON Value has some issue"));
        }

        return None
    }

    async fn get_latest_build(&self, project: &String, version: &String) -> Option<String> {
        let mut link = String::from("https://papermc.io/api/v2/projects/");
        link.push_str(&project);
        link.push_str("/versions/");
        link.push_str(&version);

        let text = reqwest::get(&link).await.unwrap().text().await.unwrap();
        let paper_json: PaperJSON = serde_json::from_str(text.as_str()).unwrap();

        if paper_json.error.is_some() {
            return None;
        }

        if paper_json.builds.is_none() {
            return None;
        }

        return Some(paper_json.builds.unwrap().iter().max().unwrap().to_string())
    }
}

#[derive(Deserialize, Default)]
struct PaperJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    builds: Option<Vec<i64>>
}