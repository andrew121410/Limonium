use std::process::exit;
use std::string::String;

use async_trait::async_trait;
use serde::de::Unexpected::Str;

use crate::api::platform;
use crate::hashutils::Hash;

// https://github.com/pufferfish-gg/Pufferfish
// https://ci.pufferfish.host/
pub struct PufferfishAPI;

#[async_trait]
impl platform::IPlatform for PufferfishAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = PufferfishAPI::get_jar_name(&self, &project, &version, &build);
        let real_version = get_real_version(&version);

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

        // Example https://ci.pufferfish.host/job/Pufferfish-1.19/lastSuccessfulBuild/artifact/build/libs/pufferfish-paperclip-1.19.2-R0.1-SNAPSHOT-reobf.jar
        let mut to_return = String::from("https://ci.pufferfish.host/job/Pufferfish-");
        to_return.push_str(&real_version.unwrap());
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

    async fn get_jar_hash(&self, _project: &String, _version: &String, _build: &String) -> Option<Hash> {
        return None;
    }
}

pub fn get_real_version(version: &String) -> Option<String> {
    if version.contains("1.19.3") {
        return Some(String::from("1.19"));
    } else if version.contains("1.18.2") {
        return Some(String::from("1.18"));
    }
    None
}

pub fn get_supported_versions() -> Vec<String> {
    return vec![
        String::from("1.19.3"),
        String::from("1.18.2"),
    ];
}