use async_trait::async_trait;

use crate::hashutils::Hash;

#[async_trait]
pub trait IPlatform: Sync {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String;

    fn get_jar_name(&self, project: &String, version: &String, build: &String) -> String;

    async fn get_latest_build(&self, project: &String, version: &String) -> Option<String>;

    async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<Hash>;
}