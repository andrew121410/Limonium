use crate::compile_controllers::TypicalSoftwareManager;
use crate::compile_controllers::TypicalSoftwareManager::SoftwareConfig;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

// CoreProtect
// coreprotect
pub(crate) struct CoreProtectAPI;

impl CoreProtectAPI {
    pub async fn handle_coreprotect(compile_path: &PathBuf, path: &mut String, branch: Option<String>, version: &str) {
        let version = version.to_string(); // Convert version to String to move it into the closure
        let config = SoftwareConfig {
            repo_url: "https://github.com/PlayPro/CoreProtect.git".to_string(),
            branch: Some("master".to_string()),
            build_command: "mvn clean package".to_string(),
            jar_regex: r"CoreProtect-\d+\.\d+\.jar".to_string(),
            jar_location: "target/".to_string(),
            before_building_function: Some(Box::new(move |software_path| {
                modify_pom_xml(software_path, &version).expect("Failed to modify pom.xml");
            })),
            custom_find_jar_function: None,
            after_building_function: None,
            delete_after_building: true
        };

        TypicalSoftwareManager::handle_software(config, compile_path, path).await;
    }
}

/// This function modifies the `pom.xml` file located in the given `software_path`.
/// It updates the first occurrence of the `<version>` tag with the provided `version`
/// and the first occurrence of the `<project.branch>` tag with "master".
///
/// # Arguments
///
/// * `software_path` - A reference to the path where the `pom.xml` file is located.
/// * `version` - A string slice that holds the new version to be set in the `pom.xml`.
///
/// # Returns
///
/// This function returns an `io::Result<()>` indicating success or failure.
fn modify_pom_xml(software_path: &PathBuf, version: &str) -> io::Result<()> {
    let pom_path = software_path.join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)?;

    let mut new_pom_content = String::new();
    let mut version_updated = false;
    let mut branch_updated = false;

    for line in pom_content.lines() {
        if !version_updated && line.trim().starts_with("<version>") && line.trim().ends_with("</version>") {
            new_pom_content.push_str(&format!("    <version>{}</version>\n", version));
            version_updated = true;
        } else if !branch_updated && line.trim().starts_with("<project.branch>") && line.trim().ends_with("</project.branch>") {
            new_pom_content.push_str("    <project.branch>development</project.branch>\n");
            branch_updated = true;
        } else {
            new_pom_content.push_str(line);
            new_pom_content.push('\n');
        }
    }

    let mut file = fs::File::create(pom_path)?;
    file.write_all(new_pom_content.as_bytes())?;

    Ok(())
}