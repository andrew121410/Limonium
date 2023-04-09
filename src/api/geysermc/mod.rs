use std::collections::HashMap;
use std::string::String;

use async_trait::async_trait;

use crate::api::bibliothek::BibliothekAPI;
use crate::api::platform;
use crate::hashutils::Hash;

static GEYSER_BIBLIOTHEK: BibliothekAPI = BibliothekAPI {
    url: "https://download.geysermc.org",
    default_channel: "standalone",
};

// https://github.com/GeyserMC/
pub struct GeyserAPI {}

#[async_trait]
impl platform::IPlatform for GeyserAPI {
    fn get_download_link(&self, project: &String, version: &String, build: &String) -> String {
        GEYSER_BIBLIOTHEK.get_download_link(&project, &version, &build)
    }

    fn get_jar_name(&self, project: &String, version: &String, build: &String) -> String {
        GEYSER_BIBLIOTHEK.get_jar_name(&project, &version, &build)
    }

    async fn get_latest_build(&self, project: &String, version: &String) -> Option<String> {
        GEYSER_BIBLIOTHEK.get_latest_build(&project, &version).await
    }

    async fn get_jar_hash(&self, project: &String, version: &String, build: &String) -> Option<Hash> {
        GEYSER_BIBLIOTHEK.get_jar_hash(&project, &version, &build).await
    }

    async fn get_latest_version(&self, project: &String) -> Option<String> {
        GEYSER_BIBLIOTHEK.get_latest_version(&project).await
    }
}

