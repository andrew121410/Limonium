use std::process::exit;
use std::string::String;

use async_trait::async_trait;
use colored::Colorize;
use regex::Regex;

use crate::download_controllers::platform;
use crate::hash_utils::Hash;
use crate::jenkins_utils;
use crate::objects::downloaded_file::DownloadedFile;

// https://github.com/pufferfish-gg/Pufferfish
// https://ci.pufferfish.host/
pub struct PufferfishAPI;

#[async_trait]
impl platform::IPlatform for PufferfishAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = PufferfishAPI::get_jar_name(&self, &project, &version, &build);
        let jenkins_version = get_jenkins_version(version);

        // Example https://ci.pufferfish.host/job/Pufferfish-1.20/lastSuccessfulBuild/artifact/build/libs/pufferfish-paperclip-1.20.4-R0.1-SNAPSHOT-reobf.jar
        let mut to_return = String::from("https://ci.pufferfish.host/job/Pufferfish-");
        to_return.push_str(&jenkins_version);
        to_return.push_str("/");
        to_return.push_str(&build);
        to_return.push_str("/artifact/build/libs/");
        to_return.push_str(&jar_name);
        return to_return;
    }

    fn get_jar_name(&self, _project: &String, version: &String, _build: &String) -> String {
        let version_number: f32 = version.split('.').take(2).collect::<Vec<&str>>().join(".").parse().unwrap_or(0.0);
        let mut to_return = String::from("pufferfish-paperclip-");
        to_return.push_str(&version);
        if version_number >= 1.21 {
            to_return.push_str("-R0.1-SNAPSHOT-mojmap.jar");
        } else {
            to_return.push_str("-R0.1-SNAPSHOT-reobf.jar");
        }
        return to_return;
    }

    async fn get_latest_build(&self, _project: &String, version: &String) -> Option<String> {
        // Verify Pufferfish Version even exists.
        let jenkins_version = get_jenkins_version(version);
        validate_jenkins_version(&jenkins_version, &version).await;

        return Some(String::from("lastSuccessfulBuild"));
    }

    // https://ci.pufferfish.host/job/Pufferfish-1.20/lastSuccessfulBuild/artifact/build/libs/pufferfish-paperclip-1.20.4-R0.1-SNAPSHOT-reobf.jar/*fingerprint*/
    // Will return a md5 hash
    async fn get_hash_from_web(&self, project: &String, version: &String, build: &String, downloaded_jar: Option<&DownloadedFile>) -> Option<Hash> {
        let jar_name = self.get_jar_name(project, version, build);
        let jenkins_version = get_jenkins_version(version);

        // Make the URL
        let mut url = String::from("https://ci.pufferfish.host/job/Pufferfish-");
        url.push_str(&jenkins_version);
        url.push_str("/");
        url.push_str(&build);
        url.push_str("/artifact/build/libs/");
        url.push_str(&jar_name);
        url.push_str("/*fingerprint*/");

        let hash = jenkins_utils::extract_file_fingerprint_hash(&url).await;
        return Some(hash);
    }

    async fn get_latest_version(&self, _project: &String) -> Option<String> {
        None
    }

    async fn custom_download_functionality(&self, _project: &String, _version: &String, _build: &String, _link: &String) -> Option<DownloadedFile> {
        None
    }
}


// Makes like https://ci.pufferfish.host/job/Pufferfish-1.20/
pub fn make_link_for_jenkins_version(jenkins_version: &String) -> String {
    let mut url = String::from("https://ci.pufferfish.host/job/Pufferfish-");
    url.push_str(jenkins_version);
    url.push_str("/");

    return url;
}

pub async fn validate_jenkins_version(jenkins_version: &String, version: &String) {
    let url = make_link_for_jenkins_version(&jenkins_version);

    // Validate the Minecraft Version associated with the Jenkins Version
    let confirm = get_minecraft_version_from_page(&url).await;

    if confirm.is_none() {
        println!("Something went wrong while trying to get the Minecraft Version does that version exist?");
        exit(1);
    }

    let compare = confirm.unwrap();

    if !compare.eq_ignore_ascii_case(version) {
        let mut build = String::from("The Minecraft Version you want (");
        build.push_str(version);
        build.push_str(") cannot be installed as only you can get the latest for that version which is ");
        build.push_str(&compare);

        println!("{}", format!("{}", build).red());
        exit(1);
    }
}

pub fn get_jenkins_version(version: &str) -> String {
    let components: Vec<&str> = version.split('.').collect();

    // Check if there are more than two components
    if components.len() > 2 {
        // Join the first two components with a dot to get the stripped version
        return components[..2].join(".");
    }

    // Return the version string unchanged
    version.to_string()
}

// Needs something like https://ci.pufferfish.host/job/Pufferfish-1.20/
// Returns latest minecraft version so if it's 1.20.4 then returns 1.20.4
pub async fn get_minecraft_version_from_page(url: &String) -> Option<String> {
    // Get the HTML
    let response = reqwest::get(url).await;
    let html = response.unwrap().text().await.unwrap();

    // Extract the MC Version using regex
    let regex = Regex::new(r#"pufferfish-paperclip-(\d+\.\d+\.\d+)"#).unwrap();
    let captures_option = regex.captures(&html);

    if captures_option.is_none() {
        return None;
    }

    let captures = captures_option.unwrap();
    let mc_version = captures.get(1).unwrap().as_str();

    return Some(mc_version.to_string());
}

#[cfg(test)]
mod pufferfish_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_minecraft_version_from_page() {
        let url = String::from("https://ci.pufferfish.host/job/Pufferfish-1.19/");
        let expected_version = String::from("1.19.4");

        let version = get_minecraft_version_from_page(&url).await;
        assert_eq!(version, Some(expected_version));

        let url = String::from("https://ci.pufferfish.host/job/Pufferfish-1.20/");
        let expected_version = String::from("1.20.4");

        let version = get_minecraft_version_from_page(&url).await;
        assert_eq!(version, Some(expected_version));
    }
}
