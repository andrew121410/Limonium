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
        let url = format!("{}/v3/projects/{}", PAPER_API_ENDPOINT, project);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .headers(limonium_headers())
            .send()
            .await
            .ok()?;
        let text = response.text().await.ok()?;

        let json: FillProjectJSON = serde_json::from_str(&text).ok()?;

        if let Some(error) = json.error {
            eprintln!("{} {}", "Error:".red(), error);
            return None;
        }

        let versions_map = json.versions?;

        // Collect both major keys and all dot versions into one list
        let mut all_versions: Vec<String> = versions_map
            .iter()
            .flat_map(|(major, versions)| {
                let mut combined = vec![major.clone()];
                combined.extend(versions.clone());
                combined
            })
            .collect();

        // Remove "-pre" versions
        all_versions.retain(|v| !v.contains("-pre"));

        // Respect the --no-snapshot-version flag
        if clap_utils::clap_get_flag_or_false("no-snapshot-version") {
            all_versions.retain(|v| !v.contains("-SNAPSHOT"));
        }

        if all_versions.is_empty() {
            eprintln!("{} {}", "Error:".red(), "No valid versions found");
            return None;
        }

        // Sort and return the latest
        number_utils::sort_versions(&mut all_versions);
        let latest_version = all_versions.last()?.to_string();

        println!("{} {}", "Latest version:".green(), &latest_version);
        Some(latest_version)
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

// https://fill.papermc.io/v3/projects/paper/versions/1.21.7
#[derive(Deserialize, Default)]
struct FillBuildsJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    builds: Option<Vec<i64>>,
}

// https://fill.papermc.io/v3/projects/paper/versions/1.21.7/builds/latest
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
