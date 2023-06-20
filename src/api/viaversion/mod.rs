use std::{env, fs};
use std::env::temp_dir;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

use async_trait::async_trait;
use colored::Colorize;
use regex::Regex;

use crate::api;
use crate::api::platform;
use crate::hashutils::Hash;

// https://github.com/ViaVersion
// https://ci.viaversion.com/
pub struct ViaVersionAPI {}

#[async_trait]
impl platform::IPlatform for ViaVersionAPI {
    async fn get_latest_version(&self, project: &String) -> Option<String> {
        Some("".to_string())
    }

    async fn get_latest_build(&self, project: &String, version: &String) -> Option<String> {
        Some("".to_string())
    }

    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let mut fallback_channel = "".to_string();
        if project.eq_ignore_ascii_case("viaversion") {
            fallback_channel = "viaversion".to_string();
        } else if project.eq_ignore_ascii_case("viabackwards") {
            fallback_channel = "viabackwards".to_string();
        }

        let channel_selected = api::get_channel_or_fallback(&fallback_channel);

        // Check if the channel is valid
        if !is_valid_channel(&channel_selected) {
            println!("{} {}", "Error:".red(), "Invalid channel selected");
            return "".to_string();
        }

        // Get the download link for the .zip file
        let mut link = get_zip_download_link(&project, &channel_selected);

        return link;
    }

    fn get_jar_name(&self, project: &String, version: &String, build: &String) -> String {
        if project.eq_ignore_ascii_case("ViaVersion") {
            return "ViaVersion.jar".to_string();
        } else if project.eq_ignore_ascii_case("ViaBackwards") {
            return "ViaBackwards.jar".to_string();
        }

        println!("{} {}", "Error:".red(), "get_jar_name() called with invalid project");
        exit(1);
    }

    async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<Hash> {
        None
    }

    async fn custom_download_functionality(&self, project: &String, version: &String, build: &String, link: &String) -> Option<String> {
        let file_name = api::jenkins_artifacts_bundle_zip_download_and_find_jar_and_place_jar_in_the_tmp_directory(&project, &version, &build, &link, r"^Via(Backwards|Version)-\d+\.\d+\.\d+(-SNAPSHOT)?\.jar$").await;

        return Some(file_name.unwrap());
    }
}

fn get_zip_download_link(project: &String, channel: &String) -> String {
    if project.eq_ignore_ascii_case("viaversion") && channel.eq_ignore_ascii_case("viaversion") {
        return "https://ci.viaversion.com/job/ViaVersion/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viaversion") && channel.eq_ignore_ascii_case("dev") {
        return "https://ci.viaversion.com/job/ViaVersion-DEV/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") && channel.eq_ignore_ascii_case("viabackwards") {
        return "https://ci.viaversion.com/view/ViaBackwards/job/ViaBackwards/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") && channel.eq_ignore_ascii_case("dev") {
        return "https://ci.viaversion.com/view/ViaBackwards/job/ViaBackwards-DEV/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    }

    return "na".to_string();
}

fn is_valid_channel(channel: &String) -> bool {
    return match channel.to_lowercase().as_str() {
        "viaversion" => true,
        "dev" => true,
        "viabackwards" => true,
        _ => false,
    };
}

fn fallback_channel(project: &String) -> String {
    if project.eq_ignore_ascii_case("viaversion") {
        return "viaversion".to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") {
        return "viabackwards".to_string();
    }

    return "".to_string();
}