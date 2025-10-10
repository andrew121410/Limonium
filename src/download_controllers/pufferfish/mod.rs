use std::process::exit;

use async_trait::async_trait;
use colored::Colorize;
use regex::Regex;
use semver::Version;

use crate::download_controllers::platform;
use crate::hash_utils::Hash;
use crate::jenkins_utils;
use crate::objects::downloaded_file::DownloadedFile;

pub struct PufferfishAPI;

#[async_trait]
impl platform::IPlatform for PufferfishAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = self.get_jar_name(project, version, build);
        let jenkins_version = get_jenkins_version(version);

        let mut to_return = format!(
            "https://ci.pufferfish.host/job/Pufferfish-{}/{}",
            jenkins_version, build
        );

        if is_new_artifact_structure(version) {
            to_return.push_str("/artifact/pufferfish-server/build/libs/");
        } else {
            to_return.push_str("/artifact/build/libs/");
        }

        to_return.push_str(&jar_name);
        to_return
    }

    fn get_jar_name(&self, _project: &String, version: &String, _build: &String) -> String {
        let suffix = if is_new_artifact_structure(version) {
            "-R0.1-SNAPSHOT-mojmap.jar"
        } else {
            "-R0.1-SNAPSHOT-reobf.jar"
        };

        format!("pufferfish-paperclip-{}{}", version, suffix)
    }

    async fn get_latest_build(&self, _project: &String, version: &String) -> Option<String> {
        let jenkins_version = get_jenkins_version(version);
        validate_jenkins_version(&jenkins_version, version).await.ok()?;
        Some("lastSuccessfulBuild".to_string())
    }

    async fn get_hash_from_web(
        &self,
        project: &String,
        version: &String,
        build: &String,
        _downloaded_jar: Option<&DownloadedFile>,
    ) -> Option<Hash> {
        let jar_name = self.get_jar_name(project, version, build);
        let jenkins_version = get_jenkins_version(version);

        let mut url = format!(
            "https://ci.pufferfish.host/job/Pufferfish-{}/{}",
            jenkins_version, build
        );

        if is_new_artifact_structure(version) {
            url.push_str("/artifact/pufferfish-server/build/libs/");
        } else {
            url.push_str("/artifact/build/libs/");
        }

        url.push_str(&jar_name);
        url.push_str("/*fingerprint*/");

        let hash = jenkins_utils::extract_file_fingerprint_hash(&url).await;
        Some(hash)
    }

    async fn get_latest_version(&self, _project: &String) -> Option<String> {
        None
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

pub fn make_link_for_jenkins_version(jenkins_version: &str) -> String {
    format!("https://ci.pufferfish.host/job/Pufferfish-{}/", jenkins_version)
}

pub async fn validate_jenkins_version(jenkins_version: &str, version: &str) -> Result<(), String> {
    let url = make_link_for_jenkins_version(jenkins_version);
    let confirm = get_minecraft_version_from_page(&url).await;

    let compare = confirm.ok_or_else(|| {
        "Something went wrong while trying to get the Minecraft Version. Does that version exist?"
            .to_string()
    })?;

    if !compare.eq_ignore_ascii_case(version) {
        let msg = format!(
            "The Minecraft Version you want ({}) cannot be installed. Only the latest for that version is available: {}",
            version, compare
        );
        println!("{}", msg.red());
        return Err(msg);
    }

    Ok(())
}

pub fn get_jenkins_version(version: &str) -> String {
    version.split('.').take(2).collect::<Vec<&str>>().join(".")
}

pub fn is_new_artifact_structure(version: &str) -> bool {
    Version::parse(version).map_or(false, |v| v >= Version::new(1, 21, 0))
}

pub async fn get_minecraft_version_from_page(url: &str) -> Option<String> {
    let response = reqwest::get(url).await.ok()?;
    let html = response.text().await.ok()?;

    let regex = Regex::new(r#"pufferfish-paperclip-(\d+\.\d+\.\d+)"#).ok()?;
    let captures = regex.captures(&html)?;

    Some(captures.get(1)?.as_str().to_string())
}

#[cfg(test)]
mod pufferfish_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_minecraft_version_from_page() {
        let url = "https://ci.pufferfish.host/job/Pufferfish-1.19/";
        let expected_version = "1.19.4";

        let version = get_minecraft_version_from_page(url).await;
        assert_eq!(version, Some(expected_version.to_string()));

        let url = "https://ci.pufferfish.host/job/Pufferfish-1.20/";
        let expected_version = "1.20.4";

        let version = get_minecraft_version_from_page(url).await;
        assert_eq!(version, Some(expected_version.to_string()));
    }

    #[test]
    fn test_is_new_artifact_structure() {
        assert!(is_new_artifact_structure("1.21.0"));
        assert!(is_new_artifact_structure("1.21.10"));
        assert!(!is_new_artifact_structure("1.20.4"));
        assert!(!is_new_artifact_structure("1.19.4"));
    }
}
