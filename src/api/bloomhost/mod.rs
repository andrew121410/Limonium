use std::collections::HashMap;
use std::string::String;

use async_trait::async_trait;
use reqwest::header;

use crate::api::platform;

// https://github.com/Bloom-host/Petal
pub struct PetalAPI;

#[async_trait]
impl platform::IPlatform for PetalAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = self.get_jar_name(&project, &version, &build);

        let status = self_update::backends::github::Update::configure()
            .repo_owner("Bloom-host")
            .repo_name("Petal")
            .target(&jar_name)
            .bin_name(&jar_name)
            .current_version("na")
            .build()
            .expect("Building failed for petal");

        let latest_release = status.get_release_version(&build).expect("Getting release version failed for petal");
        let release_asset = latest_release.asset_for(&status.target()).expect("Getting release asset failed");

        return release_asset.download_url;
    }

    fn get_jar_name(&self, _project: &String, _version: &String, _build: &String) -> String {
        return String::from("petal-1.19.jar");
    }

    // Gets the latest tag from GitHub
    async fn get_latest_build(&self, _project: &String, _version: &String) -> Option<String> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            "rust-reqwest/limonium".parse().unwrap(),
        );
        let response = reqwest::Client::new()
            .get("https://api.github.com/repos/Bloom-host/Petal/tags")
            .headers(headers)
            .send().await.unwrap();

        let text = response.text().await.unwrap();
        let tags: Vec<Tag> = serde_json::from_str(&text).expect("Failed to parse tags for petal");

        let first_wrapped = tags.first();
        if first_wrapped.is_some() {
            let first = first_wrapped.unwrap();
            return Some(first.name.clone().expect("Getting tag name failed"));
        }
        return None;
    }

    async fn get_jar_hash(&self, _project: &String, _version: &String, _build: &String) -> Option<HashMap<String, String>> {
        return None;
    }
}

#[derive(Deserialize, Default)]
struct Tag {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    name: Option<String>,
}