use std::collections::HashMap;
use std::process::exit;
use std::string::String;

use async_trait::async_trait;
use colored::Colorize;

use crate::api::platform;
use crate::githubutils::GithubUtils;

// https://github.com/Bloom-host/Petal
pub struct PetalAPI;

#[async_trait]
impl platform::IPlatform for PetalAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = self.get_jar_name(&project, &version, &build);

        let status = self_update::backends::github::Update::configure()
            .repo_owner("Bloom-host")
            .repo_name("Petal")
            .target(&jar_name)
            .bin_name("na") // Not used, but required by the API
            .current_version("na") // Not used, but required by the API
            .build()
            .expect("Building failed for petal");

        let latest_release = status.get_release_version(&build).expect("Getting release version failed for petal");
        let release_asset = latest_release.asset_for(&status.target()).expect("Getting release asset failed");

        return release_asset.download_url;
    }

    fn get_jar_name(&self, _project: &String, _version: &String, _build: &String) -> String {
        return String::from("petal-1.19.jar");
    }

    async fn get_latest_build(&self, _project: &String, version: &String) -> Option<String> {
        check_if_compatible_version(&version);

        return GithubUtils::get_latest_tag("Bloom-host", "Petal").await;
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

