use std::collections::HashMap;
use std::process::exit;
use std::string::String;

use async_trait::async_trait;
use colored::Colorize;

use crate::api::platform;

// https://github.com/Bloom-host/Petal
pub struct PetalAPI;

#[async_trait]
impl platform::IPlatform for PetalAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = self.get_jar_name(&project, &version, &build);

        return crate::githubutils::Repo::new("Bloom-host", "Petal")
            .get_download_link(&build, &jar_name);
    }

    fn get_jar_name(&self, _project: &String, version: &String, _build: &String) -> String {
        let mut jar_name = String::from("petal-");
        jar_name.push_str(&version);
        jar_name.push_str(".jar");

        return jar_name;
    }

    async fn get_latest_build(&self, _project: &String, version: &String) -> Option<String> {
        check_if_compatible_version(&version);

        return crate::githubutils::Repo::new("Bloom-host", "Petal")
            .get_latest_tag().await;
    }

    async fn get_jar_hash(&self, _project: &String, _version: &String, _build: &String) -> Option<HashMap<String, String>> {
        return None;
    }
}

fn check_if_compatible_version(version: &String) {
    if !version.eq("1.19") {
        println!("{}", format!("Version {} is not supported by Petal", version).red().bold());
        exit(1);
    }
}

