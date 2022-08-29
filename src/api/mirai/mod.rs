use std::process::exit;
use std::string::String;

use async_trait::async_trait;
use colored::Colorize;

use crate::api::platform;
use crate::hashutils::Hash;

// https://github.com/etil2jz/Mirai
pub struct MiraiAPI;

#[async_trait]
impl platform::IPlatform for MiraiAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = MiraiAPI::get_jar_name(&self, &project, &version, &build);
        let real_version = get_real_version(&version);

        // Example https://ci.codemc.io/job/etil2jz/job/Mirai-1.19/6/artifact/build/libs/mirai-paperclip-1.19.2-R0.1-SNAPSHOT-reobf.jar
        let mut to_return = String::from("https://ci.codemc.io/job/etil2jz/job/Mirai-");
        to_return.push_str(&real_version);
        to_return.push_str("/");
        to_return.push_str(&build);
        to_return.push_str("/artifact/build/libs/");
        to_return.push_str(&jar_name);
        return to_return;
    }

    fn get_jar_name(&self, _project: &String, version: &String, _build: &String) -> String {
        let mut to_return = String::from("mirai-paperclip-");
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

pub fn get_real_version(version: &String) -> String {
    if version.eq("1.19.2") {
        return String::from("1.19");
    }
    println!("{}", format!("Version {} is not supported by Mirai", version).red().bold());
    exit(1);
}