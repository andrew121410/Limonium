use async_trait::async_trait;

use crate::hash_utils::Hash;
use crate::objects::DownloadedJar::DownloadedJar;

#[async_trait]
pub trait IPlatform: Sync {
    async fn get_latest_version(&self, project: &String) -> Option<String>;

    async fn get_latest_build(&self, project: &String, version: &String) -> Option<String>;

    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String;

    fn get_jar_name(&self, project: &String, version: &String, build: &String) -> String;

    async fn get_hash_from_web(&self, project: &String, version: &String, build: &String, downloaded_jar: Option<&DownloadedJar>) -> Option<Hash>;

    async fn custom_download_functionality(&self, project: &String, version: &String, build: &String, link: &String) -> Option<DownloadedJar>;
}