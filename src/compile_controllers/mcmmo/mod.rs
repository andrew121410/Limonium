use crate::compile_controllers::TypicalSoftwareManager;
use crate::compile_controllers::TypicalSoftwareManager::SoftwareConfig;
use std::path::PathBuf;

// mcMMO
// mcmmo
pub(crate) struct mcMMOAPI;

impl mcMMOAPI {
    pub async fn handle_mcmmo(compile_path: &PathBuf, path: &mut String, branch: Option<String>) {
        let config = SoftwareConfig {
            repo_url: "https://github.com/mcMMO-Dev/mcMMO".to_string(),
            branch,
            build_command: "mvn clean package".to_string(),
            jar_regex: r"^mcMMO\.jar$".to_string(),
            jar_location: "target/".to_string(),
            before_building_function: None,
            custom_find_jar_function: None,
            after_building_function: None,
            delete_after_building: false,
        };

        TypicalSoftwareManager::handle_software(config, compile_path, path).await;
    }
}