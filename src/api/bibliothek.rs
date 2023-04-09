use std::collections::HashMap;

use std::process;
use std::rc::Rc;
use std::string::String;

use clap::ArgMatches;

use colored::Colorize;

use crate::{number_utils, SUB_COMMAND_ARG_MATCHES};

use crate::hashutils::Hash;

// https://github.com/PaperMC/bibliothek
// https://github.com/GeyserMC/bibliothek
pub struct BibliothekAPI {
    pub url: &'static str,
    pub default_channel: &'static str,
}

impl BibliothekAPI {
    pub async fn get_latest_version(&self, project: &String) -> Option<String> {
        let mut link = String::from(&self.url.to_string());
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

        // Paper for some reason has "1.13-pre-7" lol
        versions.retain(|x| !x.contains("-pre"));

        // Sort versions
        number_utils::sort_versions_one_decimal_and_two_decimal_lowest_to_highest(&mut versions);

        let latest_version: String = versions.last().unwrap().to_string();
        println!("{} {}", "Latest version:".green(), latest_version);

        Some(latest_version.to_string())
    }

    pub fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = BibliothekAPI::get_jar_name(&self, &project, &version, &build);

        let mut to_return = String::from(&self.url.to_string());
        to_return.push_str("/v2/projects/");
        to_return.push_str(&project);
        to_return.push_str("/versions/");
        to_return.push_str(&version);
        to_return.push_str("/builds/");
        to_return.push_str(&build);
        to_return.push_str("/downloads/");

        if project.eq_ignore_ascii_case("geyser") || project.eq_ignore_ascii_case("floodgate") {
            unsafe {
                let args: &Rc<ArgMatches> = SUB_COMMAND_ARG_MATCHES.as_ref().expect("SUB_COMMAND_ARG_MATCHES is not set");
                let default_channel: String = self.default_channel.to_string();
                let channel = args.get_one::<String>("channel").unwrap_or(&default_channel);
                to_return.push_str(channel);
            }
        } else {
            to_return.push_str(&jar_name);
        }

        println!("{} {}", "Download link:".green(), to_return);

        return to_return;
    }

    pub fn get_jar_name(&self, project: &String, version: &String, build: &String) -> String {
        let mut to_return = String::from("");
        to_return.push_str(&project);
        to_return.push_str("-");
        to_return.push_str(&version);
        to_return.push_str("-");
        to_return.push_str(&build);
        to_return.push_str(".jar");
        return to_return;
    }

    pub async fn get_latest_build(&self, project: &String, version: &String) -> Option<String> {
        let mut link = String::from(&self.url.to_string());
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

    pub async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<Hash> {
        let mut link = String::from(&self.url.to_string());
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

                let args: &Rc<ArgMatches> = SUB_COMMAND_ARG_MATCHES.as_ref().expect("SUB_COMMAND_ARG_MATCHES is not set");
                let default_channel: String = self.default_channel.to_string();
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

// https://api.papermc.io/v2/projects/paper
#[derive(Deserialize, Default)]
struct BibliothekProjectJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    versions: Option<Vec<String>>,
}

// https://api.papermc.io/v2/projects/paper/versions/1.19.2
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

// https://api.papermc.io/v2/projects/paper/versions/1.19.4/builds/499
// https://download.geysermc.org/v2/projects/geyser/versions/2.1.0/builds/47
#[derive(Deserialize, Default)]
struct BibliothekBuildInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    downloads: Option<HashMap<String, HashMap<String, String>>>,
}