use crate::download_controllers::{limonium_headers, platform};
use crate::hash_utils::Hash;
use crate::objects::downloaded_file::DownloadedFile;
use crate::{clap_utils, number_utils};
use async_trait::async_trait;
use colored::Colorize;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::process;
use std::string::String;

// https://github.com/PaperMC
pub struct PaperAPI {}

static PAPER_API_ENDPOINT: &'static str = "https://fill.papermc.io";
static DEFAULT_PAPER_CHANNEL: &'static str = "server:default";
static BUILD_INFO: OnceCell<FillBuildInfo> = OnceCell::new();

#[async_trait]
impl platform::IPlatform for PaperAPI {
    async fn get_latest_version(&self, project: &String) -> Option<String> {
        let mut link = String::from(&PAPER_API_ENDPOINT.to_string());
        link.push_str("/v3/projects/");
        link.push_str(&project);

        let client = reqwest::Client::new();
        let response = client
            .get(&link)
            .headers(limonium_headers())
            .send()
            .await
            .unwrap();
        let text = response.text().await.unwrap();

        let json: FillProjectJSON = serde_json::from_str(text.as_str()).unwrap();

        if json.error.is_some() {
            println!("{} {}", "Error:".red(), json.error.unwrap());
            return None;
        }

        if json.versions.is_none() {
            println!("{} {}", "Error:".red(), "No versions found");
            return None;
        }

        let mut versions: HashMap<String, Vec<String>> = json.versions.unwrap();

        // HashMap keys are the major_versions of a version so like (1.19, 1.20, 1.21) and then the values are the dot versions like 1.21.1, 1.21.2, etc.
        // Essentially we need to get the highest major version and then the highest dot version of that major version.

        let mut major_versions: Vec<String> = versions.keys().cloned().collect();

        if major_versions.is_empty() {
            println!(
                "{} {}",
                "Error:".red(),
                "No versions found (keys are empty)"
            );
            return None;
        }

        // Sort major versions
        number_utils::sort_versions(&mut major_versions);

        let highest_major_version: String = major_versions.last().unwrap().to_string();
        let mut dot_versions: Vec<String> = versions.get(&highest_major_version).unwrap().to_vec();

        // Sort dot versions
        number_utils::sort_versions(&mut dot_versions);

        // Paper for some reason has "1.13-pre-7" lol
        dot_versions.retain(|x| !x.contains("-pre"));

        // See if we don't include snapshot versions
        let dont_include_snapshot_versions: bool =
            clap_utils::clap_get_flag_or_false(&String::from("no-snapshot-version"));
        if dont_include_snapshot_versions {
            dot_versions.retain(|x| !x.contains("-SNAPSHOT"));
        }

        let latest_version: String = dot_versions.last().unwrap().to_string();

        println!("{} {}", "Latest version:".green(), latest_version);
        Some(latest_version.to_string())
    }

    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let cache = BUILD_INFO.get().expect("Build info not cached");

        let downloads = &cache.downloads;
        let channel = clap_utils::clap_get_one_or_fallback(
            &String::from("channel"),
            &String::from(DEFAULT_PAPER_CHANNEL),
        );

        let download_info = downloads.get(&channel).unwrap();
        let download_url = &download_info.url;
        download_url.to_string()
    }

    fn get_jar_name(&self, project: &String, version: &String, build: &String) -> String {
        let mut to_return = String::from("");
        to_return.push_str(&project);
        to_return.push_str("-");
        to_return.push_str(&version);
        to_return.push_str("-");
        to_return.push_str(&build);
        to_return.push_str(".jar");
        to_return
    }

    async fn get_latest_build(&self, project: &String, version: &String) -> Option<String> {
        // Glad Paper brought back the "latest" build endpoint
        Some("latest".to_string())
    }

    async fn get_hash_from_web(
        &self,
        project: &String,
        version: &String,
        build: &String,
        downloaded_jar: Option<&DownloadedFile>,
    ) -> Option<Hash> {
        let mut link = String::from(&PAPER_API_ENDPOINT.to_string());
        link.push_str("/v3/projects/");
        link.push_str(&project);
        link.push_str("/versions/");
        link.push_str(&version);
        link.push_str("/builds/");
        link.push_str(&build);

        let client = reqwest::Client::new();
        let response = client
            .get(&link)
            .headers(limonium_headers())
            .send()
            .await
            .unwrap();
        let text = response.text().await.unwrap();

        let paper_build_info_json: FillBuildInfo = serde_json::from_str(text.as_str()).unwrap();

        // Save in cache for later use
        let _ = BUILD_INFO.set(paper_build_info_json.clone());

        if paper_build_info_json.downloads.is_empty() {
            return None;
        }

        let downloads = paper_build_info_json.downloads;

        let channel = clap_utils::clap_get_one_or_fallback(
            &String::from("channel"),
            &String::from(DEFAULT_PAPER_CHANNEL),
        );

        if !downloads.contains_key(&channel) {
            println!("{} channel does not exist", format!("{}", channel).red());
            list_all_available_channels(project, version, build).await;
            process::exit(102);
        }

        let sha256: &String = downloads
            .get(&channel)
            .unwrap()
            .checksums
            .get("sha256")
            .unwrap();

        Some(Hash::new(String::from("sha256"), sha256.clone()))
    }

    async fn custom_download_functionality(
        &self,
        _project: &String,
        _version: &String,
        _build: &String,
        _link: &String,
    ) -> Option<DownloadedFile> {
        None
    }
}

async fn list_all_available_channels(project: &String, version: &String, build: &String) {
    let mut link = String::from(&PAPER_API_ENDPOINT.to_string());
    link.push_str("/v3/projects/");
    link.push_str(&project);
    link.push_str("/versions/");
    link.push_str(&version);
    link.push_str("/builds/");
    link.push_str(&build);

    let client = reqwest::Client::new();
    let response = client
        .get(&link)
        .headers(limonium_headers())
        .send()
        .await
        .unwrap();
    let text = response.text().await.unwrap();

    let paper_build_info_json: FillBuildInfo = serde_json::from_str(text.as_str()).unwrap();

    let downloads = paper_build_info_json.downloads;
    println!(
        "{} {}",
        "Available channels:".green(),
        downloads
            .keys()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>()
            .join(", ")
    );
}

// https://fill.papermc.io/v3/projects/paper
#[derive(Deserialize, Default)]
struct FillProjectJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    versions: Option<HashMap<String, Vec<String>>>,
}

// https://fill.papermc.io/v3/projects/paper/versions/1.20.5
#[derive(Deserialize, Default)]
struct FillBuildsJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    builds: Option<Vec<i64>>,
}

// https://fill.papermc.io/v3/projects/paper/versions/1.20.5/builds/latest
#[derive(Deserialize, Default, Clone)]
struct FillBuildInfo {
    id: i32,
    time: String,
    channel: String,
    downloads: HashMap<String, FillDownloadChannel>,
}
#[derive(Deserialize, Default, Clone)]
struct FillDownloadChannel {
    name: String,
    checksums: HashMap<String, String>,
    size: u64,
    url: String,
}
