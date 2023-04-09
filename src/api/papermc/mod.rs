
use std::string::String;

use async_trait::async_trait;

use crate::api::bibliothek::BibliothekAPI;
use crate::api::platform;
use crate::hashutils::Hash;

static PAPER_BIBLIOTHEK: BibliothekAPI = BibliothekAPI {
    url: "https://api.papermc.io",
    default_channel: "application",
};

// https://github.com/PaperMC
pub struct PaperAPI {}

#[async_trait]
impl platform::IPlatform for PaperAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        PAPER_BIBLIOTHEK.get_download_link(&project, &version, &build)
    }

    fn get_jar_name(&self, project: &String, version: &String, build: &String) -> String {
        PAPER_BIBLIOTHEK.get_jar_name(&project, &version, &build)
    }

    async fn get_latest_build(&self, project: &String, version: &String) -> Option<String> {
        PAPER_BIBLIOTHEK.get_latest_build(&project, &version).await
    }

    async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<Hash> {
        PAPER_BIBLIOTHEK.get_jar_hash(&project, &version, &build).await
    }

    async fn get_latest_version(&self, project: &String) -> Option<String> {
        PAPER_BIBLIOTHEK.get_latest_version(&project).await
    }
}

