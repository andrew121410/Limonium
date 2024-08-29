use crate::compile_controllers::TypicalSoftwareManager;
use crate::compile_controllers::TypicalSoftwareManager::SoftwareConfig;
use std::path::PathBuf;

// PlotSquared
// plotsquared
pub(crate) struct PlotSquaredAPI;

impl PlotSquaredAPI {
    pub async fn handle_plotsquared(compile_path: &PathBuf, path: &mut String, branch: Option<String>) {
        let config = SoftwareConfig {
            repo_url: "https://github.com/IntellectualSites/PlotSquared.git".to_string(),
            branch,
            build_command: "gradlew build".to_string(),
            jar_regex: r"plotsquared-.*\.jar".to_string(),
            jar_location: "Bukkit/build/libs/".to_string(),
            before_building_function: Some(Box::new(|software_path| {
                // Remove withJavadocJar() from build.gradle.kts
                let build_gradle_path = software_path.join("build.gradle.kts");
                let build_gradle = std::fs::read_to_string(&build_gradle_path).unwrap();
                let new_build_gradle = build_gradle.replace("withJavadocJar()", "");
                std::fs::write(&build_gradle_path, new_build_gradle).unwrap();
                println!("Removed withJavadocJar() from build.gradle.kts");
            })),
            custom_find_jar_function: None,
            after_building_function: None,
        };

        TypicalSoftwareManager::handle_software(config, compile_path, path).await;
    }
}