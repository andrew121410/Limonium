use crate::compile_controllers::TypicalSoftwareManager;
use crate::compile_controllers::TypicalSoftwareManager::SoftwareConfig;
use std::path::PathBuf;

// PlotSquared
// plotsquared
pub(crate) struct PlotSquaredAPI;

impl PlotSquaredAPI {
    pub async fn handle_plotsquared(compile_path: &PathBuf, path: &mut String) {
        let config = SoftwareConfig {
            repo_url: "https://github.com/IntellectualSites/PlotSquared.git".to_string(),
            branch: None,
            build_command: "gradlew build".to_string(),
            jar_regex: r"plotsquared-.*\.jar".to_string(),
            jar_location: "Bukkit/build/libs/".to_string(),
        };

        TypicalSoftwareManager::handle_software(config, compile_path, path).await;
    }
}