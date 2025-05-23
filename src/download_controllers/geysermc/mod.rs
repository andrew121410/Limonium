use std::collections::HashMap;
use std::process;
use std::string::String;

use async_trait::async_trait;
use colored::Colorize;

use crate::{clap_utils, download_controllers, number_utils};
use crate::download_controllers::platform;
use crate::hash_utils::Hash;
use crate::objects::downloaded_file::DownloadedFile;

// https://github.com/GeyserMC/
pub struct GeyserAPI {}

static GEYSER_API_ENDPOINT: &'static str = "https://download.geysermc.org";
static DEFAULT_GEYSER_CHANNEL: &'static str = "standalone";

#[async_trait]
impl platform::IPlatform for GeyserAPI {
    async fn get_latest_version(&self, project: &String) -> Option<String> {
        let mut link = String::from(&GEYSER_API_ENDPOINT.to_string());
        link.push_str("/v2/projects/");
        link.push_str(&project);

        let text = reqwest::get(&link).await.unwrap().text().await.unwrap();
        let json: BibliothekProjectJSON = serde_json::from_str(text.as_str()).unwrap();

        if json.error.is_some() {
            println!("{} {}", "Error:".red(), json.error.unwrap());
            return None;
        }

        if json.versions.is_none() {
            println!("{} {}", "Error:".red(), "No versions found");
            return None;
        }

        let mut versions: Vec<String> = json.versions.unwrap();

        // See if we don't include snapshot versions
        let dont_include_snapshot_versions: bool = clap_utils::clap_get_flag_or_false(&String::from("no-snapshot-version"));
        if dont_include_snapshot_versions {
            versions.retain(|x| !x.contains("-SNAPSHOT"));
        }

        // Sort versions
        number_utils::sort_versions(&mut versions);

        let latest_version: String = versions.last().unwrap().to_string();

        println!("{} {}", "Latest version:".green(), latest_version);
        Some(latest_version.to_string())
    }

    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let mut to_return = String::from(&GEYSER_API_ENDPOINT.to_string());
        to_return.push_str("/v2/projects/");
        to_return.push_str(&project);
        to_return.push_str("/versions/");
        to_return.push_str(&version);
        to_return.push_str("/builds/");
        to_return.push_str(&build);
        to_return.push_str("/downloads/");

        let channel = clap_utils::clap_get_one_or_fallback(&String::from("channel"), &String::from(DEFAULT_GEYSER_CHANNEL));
        to_return.push_str(&channel);

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
        let mut link = String::from(&GEYSER_API_ENDPOINT.to_string());
        link.push_str("/v2/projects/");
        link.push_str(&project);
        link.push_str("/versions/");
        link.push_str(&version);

        let text = reqwest::get(&link).await.unwrap().text().await.unwrap();
        let geyser_json: BibliothekBuildsJSON = serde_json::from_str(text.as_str()).unwrap();

        if geyser_json.error.is_some() {
            return None;
        }

        if geyser_json.builds.is_none() {
            return None;
        }

        let latest_build: String = geyser_json.builds.unwrap().iter().max().unwrap().to_string();
        return Some(latest_build);
    }

    async fn get_hash_from_web(&self, project: &String, version: &String, build: &String, downloaded_jar: Option<&DownloadedFile>) -> Option<Hash> {
        let mut link = String::from(&GEYSER_API_ENDPOINT.to_string());
        link.push_str("/v2/projects/");
        link.push_str(&project);
        link.push_str("/versions/");
        link.push_str(&version);
        link.push_str("/builds/");
        link.push_str(&build);

        let text = reqwest::get(&link).await.unwrap().text().await.unwrap();
        let geyser_build_info_json: BibliothekBuildInfo = serde_json::from_str(text.as_str()).unwrap();

        if geyser_build_info_json.downloads.is_some() {
            let downloads = geyser_build_info_json.downloads.unwrap();
            let channel = clap_utils::clap_get_one_or_fallback(&String::from("channel"), &String::from(DEFAULT_GEYSER_CHANNEL));

            // Check if channel exists
            if !downloads.contains_key(&channel) {
                println!("{} channel does not exist", format!("{}", channel).red());
                list_all_available_channels(project, version, build).await;
                process::exit(102);
            }

            let sha256: &String = downloads.get(&channel).unwrap().get("sha256").unwrap();

            return Some(Hash::new(String::from("sha256"), sha256.clone()));
        }
        return None;
    }

    async fn custom_download_functionality(&self, _project: &String, _version: &String, _build: &String, _link: &String) -> Option<DownloadedFile> {
        None
    }
}

async fn list_all_available_channels(project: &String, version: &String, build: &String) {
    let mut link = String::from(&GEYSER_API_ENDPOINT.to_string());
    link.push_str("/v2/projects/");
    link.push_str(&project);
    link.push_str("/versions/");
    link.push_str(&version);
    link.push_str("/builds/");
    link.push_str(&build);

    let text = reqwest::get(&link).await.unwrap().text().await.unwrap();
    let geyser_build_info_json: BibliothekBuildInfo = serde_json::from_str(text.as_str()).unwrap();

    if geyser_build_info_json.downloads.is_some() {
        let downloads = geyser_build_info_json.downloads.unwrap();

        println!("{} {}", "Available channels:".green(), downloads.keys().map(|x| format!("{}", x)).collect::<Vec<String>>().join(", "));
    }
}

// https://download.geysermc.org/v2/projects/geyser
#[derive(Deserialize, Default)]
struct BibliothekProjectJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    versions: Option<Vec<String>>,
}

// https://download.geysermc.org/v2/projects/geyser/versions/2.1.0
#[derive(Deserialize, Default)]
struct BibliothekBuildsJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    builds: Option<Vec<i64>>,
}

// https://download.geysermc.org/v2/projects/geyser/versions/2.1.0/builds/47
#[derive(Deserialize, Default)]
struct BibliothekBuildInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    downloads: Option<HashMap<String, HashMap<String, String>>>,
}