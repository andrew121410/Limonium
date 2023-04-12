use std::collections::HashMap;
use std::process;
use std::string::String;

use async_trait::async_trait;
use clap::ArgMatches;
use colored::Colorize;

use crate::{number_utils, SUB_COMMAND_ARG_MATCHES};
use crate::api::platform;
use crate::hashutils::Hash;

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
        unsafe {
            let args: &ArgMatches = SUB_COMMAND_ARG_MATCHES.as_ref().expect("SUB_COMMAND_ARG_MATCHES is not set");
            let dont_include_snapshot_versions: bool = args.get_flag("latest-dont-include-snapshot-versions");
            if dont_include_snapshot_versions {
                versions.retain(|x| !x.contains("-SNAPSHOT"));
            }
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

        unsafe {
            let args: &ArgMatches = SUB_COMMAND_ARG_MATCHES.as_ref().expect("SUB_COMMAND_ARG_MATCHES is not set");
            let default_channel: String = DEFAULT_GEYSER_CHANNEL.to_string();
            let channel = args.get_one::<String>("channel").unwrap_or(&default_channel);
            to_return.push_str(channel);
        }

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
        let paper_json: BibliothekBuildsJSON = serde_json::from_str(text.as_str()).unwrap();

        if paper_json.error.is_some() {
            return None;
        }

        if paper_json.builds.is_none() {
            return None;
        }

        let latest_build: String = paper_json.builds.unwrap().iter().max().unwrap().to_string();
        return Some(latest_build);
    }

    async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<Hash> {
        let mut link = String::from(&GEYSER_API_ENDPOINT.to_string());
        link.push_str("/v2/projects/");
        link.push_str(&project);
        link.push_str("/versions/");
        link.push_str(&version);
        link.push_str("/builds/");
        link.push_str(&build);

        let text = reqwest::get(&link).await.unwrap().text().await.unwrap();
        let paper_build_info_json: BibliothekBuildInfo = serde_json::from_str(text.as_str()).unwrap();

        unsafe {
            if paper_build_info_json.downloads.is_some() {
                let downloads = paper_build_info_json.downloads.unwrap();

                let args: &ArgMatches = SUB_COMMAND_ARG_MATCHES.as_ref().expect("SUB_COMMAND_ARG_MATCHES is not set");
                let default_channel: String = DEFAULT_GEYSER_CHANNEL.to_string();
                let channel = args.get_one::<String>("channel").unwrap_or(&default_channel);

                if !downloads.contains_key(channel) {
                    println!("{} channel does not exist", format!("{}", channel).red());
                    process::exit(102);
                }

                let sha256: &String = downloads.get(channel).unwrap().get("sha256").unwrap();

                return Some(Hash::new(String::from("sha256"), sha256.clone()));
            }
        }
        return None;
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