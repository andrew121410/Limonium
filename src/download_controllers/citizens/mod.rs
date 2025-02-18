use std::string::String;

use async_trait::async_trait;

use crate::download_controllers::platform;
use crate::hash_utils::Hash;
use crate::jenkins_utils;
use crate::objects::DownloadedJar::DownloadedJar;

// https://github.com/CitizensDev/Citizens2
// https://ci.citizensnpcs.co/job/Citizens2/
pub struct Citizens2API;

#[async_trait]
impl platform::IPlatform for Citizens2API {
    async fn get_latest_version(&self, _project: &String) -> Option<String> {
        Some("".to_string())
    }

    async fn get_latest_build(&self, _project: &String, _version: &String) -> Option<String> {
        Some("".to_string())
    }

    fn get_download_link(&self, project: &String, _version: &String, _build: &String) -> String {
        "https://ci.citizensnpcs.co/job/Citizens2/lastSuccessfulBuild/artifact/dist/target/*zip*/target.zip".to_string()
    }

    fn get_jar_name(&self, _project: &String, version: &String, _build: &String) -> String {
        "Citizens.jar".to_string()
    }

    async fn get_hash_from_web(
        &self,
        project: &String,
        version: &String,
        build: &String,
        downloaded_jar_option: Option<&DownloadedJar>,
    ) -> Option<Hash> {
        if downloaded_jar_option.is_none() {
            return None;
        }

        let downloaded_jar = downloaded_jar_option.unwrap();

        // We must have the real jar name.
        if downloaded_jar.real_jar_name.is_none() {
            return None;
        }

        let jar_name = downloaded_jar.real_jar_name.as_ref().unwrap();
        // https://ci.citizensnpcs.co/job/Citizens2/lastSuccessfulBuild/artifact/dist/target/Citizens-2.0.37-b3714.jar/*fingerprint*/
        let fingerprint_link = format!("https://ci.citizensnpcs.co/job/Citizens2/lastSuccessfulBuild/artifact/dist/target/{}/*fingerprint*/", jar_name);
        let hash = jenkins_utils::extract_file_fingerprint_hash(&fingerprint_link).await;
        Some(hash)
    }

    async fn custom_download_functionality(
        &self,
        project: &String,
        version: &String,
        build: &String,
        link: &String,
    ) -> Option<DownloadedJar> {
        let downloaded_jar_option: Option<DownloadedJar> =
            jenkins_utils::download_and_extract_jenkins_artifact(
                &project,
                &version,
                &build,
                &link,
                r"^Citizens-\d+\.\d+\.\d+-b\d+\.jar$",
            )
            .await;

        if downloaded_jar_option.is_none() {
            println!("Error: Citizens (custom_download_functionality) failed to download the jar");
            return None;
        }

        Some(downloaded_jar_option.unwrap())
    }
}
