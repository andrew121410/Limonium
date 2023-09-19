use std::string::String;

use async_trait::async_trait;

use crate::controllers::platform;
use crate::hash_utils::Hash;
use crate::jenkins_utils;
use crate::objects::DownloadedJar::DownloadedJar;

// https://github.com/SpigotMC/BungeeCord
// https://ci.md-5.net/job/BungeeCord/
pub struct BungeeCordAPI;

#[async_trait]
impl platform::IPlatform for BungeeCordAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        let jar_name = BungeeCordAPI::get_jar_name(&self, &project, &version, &build);

        // Example https://ci.md-5.net/job/BungeeCord/lastSuccessfulBuild/artifact/bootstrap/target/BungeeCord.jar
        let mut to_return = String::from("https://ci.md-5.net/job/BungeeCord/");
        to_return.push_str(&build);
        to_return.push_str("/artifact/bootstrap/target/");
        to_return.push_str(&jar_name);
        return to_return;
    }

    fn get_jar_name(&self, _project: &String, version: &String, _build: &String) -> String {
        return "BungeeCord.jar".to_string();
    }

    async fn get_latest_build(&self, _project: &String, _version: &String) -> Option<String> {
        return Some(String::from("lastSuccessfulBuild"));
    }

    // https://ci.md-5.net/job/BungeeCord/lastSuccessfulBuild/artifact/bootstrap/target/BungeeCord.jar/*fingerprint*/
    // Will return a md5 hash
    async fn get_hash_from_web(&self, project: &String, version: &String, build: &String, downloaded_jar: Option<&DownloadedJar>) -> Option<Hash> {
        let jar_name = BungeeCordAPI::get_jar_name(&self, &project, &version, &build);

        // Make the url
        let mut url = String::from("https://ci.md-5.net/job/BungeeCord/");
        url.push_str(&build);
        url.push_str("/artifact/bootstrap/target/");
        url.push_str(&jar_name);
        url.push_str("/*fingerprint*/");

        // Get the hash
        let hash = jenkins_utils::extract_file_fingerprint_hash(&url).await;
        return Some(hash);
    }

    async fn get_latest_version(&self, _project: &String) -> Option<String> {
        return Some(String::from(""));
    }

    async fn custom_download_functionality(&self, _project: &String, _version: &String, _build: &String, _link: &String) -> Option<DownloadedJar> {
        None
    }
}