use std::ascii::AsciiExt;
use std::process::exit;

use async_trait::async_trait;
use colored::Colorize;

use crate::{download_controllers, jenkins_utils};
use crate::download_controllers::platform;
use crate::hash_utils::Hash;
use crate::objects::DownloadedJar::DownloadedJar;

// https://github.com/ViaVersion
// https://ci.viaversion.com/
pub struct ViaVersionAPI {}

#[async_trait]
impl platform::IPlatform for ViaVersionAPI {
    async fn get_latest_version(&self, _project: &String) -> Option<String> {
        Some("".to_string())
    }

    async fn get_latest_build(&self, _project: &String, _version: &String) -> Option<String> {
        Some("".to_string())
    }

    fn get_download_link(&self, project: &String, _version: &String, _build: &String) -> String {
        let fallback_channel = fallback_channel(&project);
        let channel_selected = download_controllers::clap_get_one_or_fallback(&"channel".to_string(), &fallback_channel);

        // Check if the channel is valid
        if !is_valid_channel(&channel_selected) {
            println!("{} {}", "Error:".red(), "Invalid channel selected");
            return "".to_string();
        }

        // Get the download link for the .zip file
        let link = get_zip_download_link(&project, &channel_selected);

        return link;
    }

    fn get_jar_name(&self, project: &String, _version: &String, _build: &String) -> String {
        if project.eq_ignore_ascii_case("ViaVersion") {
            return "ViaVersion.jar".to_string();
        } else if project.eq_ignore_ascii_case("ViaBackwards") {
            return "ViaBackwards.jar".to_string();
        }

        println!("{} {}", "Error:".red(), "get_jar_name() called with invalid project");
        exit(1);
    }

    async fn get_hash_from_web(&self, project: &String, version: &String, build: &String, downloaded_jar_option: Option<&DownloadedJar>) -> Option<Hash> {
        if downloaded_jar_option.is_none() {
            return None;
        }

        let fallback_channel = fallback_channel(&project);
        let channel_selected = download_controllers::clap_get_one_or_fallback(&"channel".to_string(), &fallback_channel);

        // Check if the channel is valid
        if !is_valid_channel(&channel_selected) {
            println!("{} {}", "Error:".red(), "Invalid channel selected");
            return None;
        }

        let downloaded_jar = downloaded_jar_option.unwrap();

        // We must have the real jar name.
        if downloaded_jar.real_jar_name.is_none() {
            return None;
        }

        let jar_name = downloaded_jar.real_jar_name.as_ref().unwrap();

        let fingerprint_link = get_fingerprint_link(&project, &channel_selected, &jar_name);

        let hash = jenkins_utils::extract_file_fingerprint_hash(&fingerprint_link).await;

        return Some(hash);
    }

    async fn custom_download_functionality(&self, project: &String, version: &String, build: &String, link: &String) -> Option<DownloadedJar> {
        let downloaded_jar_option: Option<DownloadedJar> = jenkins_utils::jenkins_artifacts_bundle_zip_download_and_find_jar_and_place_jar_in_the_tmp_directory(
            &project,
            &version,
            &build,
            &link,
            r"^Via(Backwards|Version)-\d+\.\d+\.\d+(-SNAPSHOT)?(-downgraded)?\.jar$").await; // r"^Via(Backwards|Version)-\d+\.\d+\.\d+(-SNAPSHOT)?\.jar$

        if downloaded_jar_option.is_none() {
            println!("{} {}", "Error:".red(), "ViaVersion (custom_download_functionality) failed to download the jar");
            exit(101);
        }

        return Some(downloaded_jar_option.unwrap());
    }
}

fn get_zip_download_link(project: &String, channel: &String) -> String {
    if project.eq_ignore_ascii_case("viaversion") && channel.eq_ignore_ascii_case("viaversion") {
        return "https://ci.viaversion.com/job/ViaVersion/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viaversion") && channel.eq_ignore_ascii_case("dev") {
        return "https://ci.viaversion.com/job/ViaVersion-DEV/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viaversion") && channel.eq_ignore_ascii_case("compatibility") {
        return "https://ci.viaversion.com/job/ViaVersion-Java8/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") && channel.eq_ignore_ascii_case("viabackwards") {
        return "https://ci.viaversion.com/view/ViaBackwards/job/ViaBackwards/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") && channel.eq_ignore_ascii_case("dev") {
        return "https://ci.viaversion.com/view/ViaBackwards/job/ViaBackwards-DEV/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") && channel.eq_ignore_ascii_case("compatibility") {
        return "https://ci.viaversion.com/view/ViaBackwards/job/ViaBackwards-Java8/lastSuccessfulBuild/artifact/build/libs/*zip*/libs.zip".to_string();
    }

    return "na".to_string();
}

fn get_fingerprint_link(project: &String, channel: &String, jar_name: &String) -> String {
    if project.eq_ignore_ascii_case("viaversion") && channel.eq_ignore_ascii_case("viaversion") {
        let mut link = "https://ci.viaversion.com/job/ViaVersion/lastSuccessfulBuild/artifact/build/libs/".to_string();
        link.push_str(jar_name);
        link.push_str("/*fingerprint*/");
        return link.to_string();
    } else if project.eq_ignore_ascii_case("viaversion") && channel.eq_ignore_ascii_case("dev") {
        let mut link = "https://ci.viaversion.com/job/ViaVersion-DEV/lastSuccessfulBuild/artifact/build/libs/".to_string();
        link.push_str(jar_name);
        link.push_str("/*fingerprint*/");
        return link.to_string();
    } else if project.eq_ignore_ascii_case("viaversion") && channel.eq_ignore_ascii_case("compatibility") {
        let mut link = "https://ci.viaversion.com/job/ViaVersion-Java8/lastSuccessfulBuild/artifact/build/libs/".to_string();
        link.push_str(jar_name);
        link.push_str("/*fingerprint*/");
        return link.to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") && channel.eq_ignore_ascii_case("viabackwards") {
        let mut link = "https://ci.viaversion.com/job/ViaBackwards/lastSuccessfulBuild/artifact/build/libs/".to_string();
        link.push_str(jar_name);
        link.push_str("/*fingerprint*/");
        return link.to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") && channel.eq_ignore_ascii_case("dev") {
        let mut link = "https://ci.viaversion.com/job/ViaBackwards-DEV/lastSuccessfulBuild/artifact/build/libs/".to_string();
        link.push_str(jar_name);
        link.push_str("/*fingerprint*/");
        return link.to_string();
    } else if project.eq_ignore_ascii_case("viabackwards") && channel.eq_ignore_ascii_case("compatibility") {
        let mut link = "https://ci.viaversion.com/view/ViaBackwards/job/ViaBackwards-Java8/lastSuccessfulBuild/artifact/build/libs/".to_string();
        link.push_str(jar_name);
        link.push_str("/*fingerprint*/");
        return link.to_string();
    }

    return "na".to_string();
}

fn is_valid_channel(channel: &String) -> bool {
    return match channel.to_lowercase().as_str() {
        "viaversion" => true,
        "dev" => true,
        "viabackwards" => true,
        "compatibility" => true,
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