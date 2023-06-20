use async_trait::async_trait;

use crate::hashutils::Hash;

#[async_trait]
pub trait IPlatform: Sync {
    async fn get_latest_version(&self, project: &String) -> Option<String>;

    async fn get_latest_build(&self, project: &String, version: &String) -> Option<String>;

    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String;

    fn get_jar_name(&self, project: &String, version: &String, build: &String) -> String;

    async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<Hash>;

    // Returns file name in the /tmp directory (None if don't want to override the download functionality)
    async fn custom_download_functionality(&self, project: &String, version: &String, build: &String, link: &String) -> Option<String>;
}