use std::process::exit;
use std::string::String;

use async_trait::async_trait;
use regex::Regex;

use crate::api::platform;
use crate::hashutils::Hash;

// https://github.com/pufferfish-gg/Pufferfish
// https://ci.pufferfish.host/
pub struct PufferfishAPI;

#[async_trait]
impl platform::IPlatform for PufferfishAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = PufferfishAPI::get_jar_name(&self, &project, &version, &build);
        let jenkins_version = get_jenkins_version(&version);
        validate_jenkins_version(&jenkins_version, &version);

        // Example https://ci.pufferfish.host/job/Pufferfish-1.19/lastSuccessfulBuild/artifact/build/libs/pufferfish-paperclip-1.19.2-R0.1-SNAPSHOT-reobf.jar
        let mut to_return = String::from("https://ci.pufferfish.host/job/Pufferfish-");
        to_return.push_str(&jenkins_version.unwrap());
        to_return.push_str("/");
        to_return.push_str(&build);
        to_return.push_str("/artifact/build/libs/");
        to_return.push_str(&jar_name);
        return to_return;
    }

    fn get_jar_name(&self, _project: &String, version: &String, _build: &String) -> String {
        let mut to_return = String::from("pufferfish-paperclip-");
        to_return.push_str(&version);
        to_return.push_str("-R0.1-SNAPSHOT-reobf.jar");
        return to_return;
    }

    async fn get_latest_build(&self, _project: &String, _version: &String) -> Option<String> {
        return Some(String::from("lastSuccessfulBuild"));
    }

    // https://ci.pufferfish.host/job/Pufferfish-1.20/lastSuccessfulBuild/artifact/build/libs/pufferfish-paperclip-1.20.1-R0.1-SNAPSHOT-reobf.jar/*fingerprint*/
    // Will return a md5 hash
    async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<Hash> {
        let jar_name = self.get_jar_name(project, version, build);
        let jenkins_version = get_jenkins_version(version);
        validate_jenkins_version(&jenkins_version, &version);

        // Make the URL
        let mut url = String::from("https://ci.pufferfish.host/job/Pufferfish-");
        url.push_str(&jenkins_version.unwrap());
        url.push_str("/");
        url.push_str(&build);
        url.push_str("/artifact/build/libs/");
        url.push_str(&jar_name);
        url.push_str("/*fingerprint*/");

        // Get the HTML
        let response = reqwest::get(&url).await;
        let html = response.unwrap().text().await.unwrap();

        // Extract the MD5 hash using regex
        let re = Regex::new(r#"The fingerprint (\w{32})"#).unwrap();
        let captures = re.captures(&html).expect("Failed to extract MD5 hash");
        let md5_hash = captures.get(1).unwrap().as_str();

        let hash = Hash {
            algorithm: String::from("md5"),
            hash: String::from(md5_hash),
        };

        return Some(hash);
    }

    async fn get_latest_version(&self, _project: &String) -> Option<String> {
        None
    }

    async fn custom_download_functionality(&self, _project: &String, _version: &String, _build: &String, _link: &String) -> Option<String> {
        None
    }
}

pub fn validate_jenkins_version(real_version: &Option<String>, version: &String) {
    if real_version.is_none() {
        println!("Pufferfish: Invalid version: {}", version);
        let supported_versions = get_supported_versions();
        println!();
        println!("Supported versions:");
        for version in supported_versions {
            println!("  {}", version);
        }
        exit(1)
    }
}

pub fn get_jenkins_version(version: &String) -> Option<String> {
    if version.contains("1.20.1") {
        return Some(String::from("1.20"));
    } else if version.contains("1.19.4") {
        return Some(String::from("1.19"));
    } else if version.contains("1.18.2") {
        return Some(String::from("1.18"));
    }
    None
}

pub fn get_supported_versions() -> Vec<String> {
    return vec![
        String::from("1.20.1"),
        String::from("1.19.4"),
        String::from("1.18.2"),
    ];
}