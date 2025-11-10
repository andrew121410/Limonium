use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use uuid::Uuid;
use std::sync::OnceLock;
use colored::Colorize;

// Store the unique directory path for this instance
static INSTANCE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Gets or creates a unique limonium directory for this instance
/// Each instance gets its own subdirectory to avoid conflicts
pub fn get_or_create_limonium_dir() -> PathBuf {
    let dir = INSTANCE_DIR.get_or_init(|| {
        let mut base_dir = env::temp_dir();
        base_dir.push("limonium");

        // Create a unique subdirectory for this instance using UUID
        let instance_id = Uuid::new_v4().to_string();
        base_dir.push(instance_id);

        base_dir
    });

    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create instance directory");
    }

    dir.clone()
}

/// Deletes only this instance's temporary folder
pub fn delete_limonium_folder() -> std::io::Result<()> {
    if let Some(dir) = INSTANCE_DIR.get() {
        if dir.exists() {
            println!("{}", "Cleaning up temporary directory...".yellow());
            fs::remove_dir_all(&dir)?;
        }
    }

    Ok(())
}

/// Perform cleanup - deletes this instance's directory and old directories
pub fn cleanup() {
    let _ = delete_limonium_folder();
    let _ = cleanup_old_instance_dirs();
}

/// Clean up old instance directories (older than 24 hours) to prevent accumulation
fn cleanup_old_instance_dirs() -> std::io::Result<()> {
    let mut base_dir = env::temp_dir();
    base_dir.push("limonium");

    if !base_dir.exists() {
        return Ok(());
    }

    let now = std::time::SystemTime::now();
    let one_day = std::time::Duration::from_secs(24 * 60 * 60);

    if let Ok(entries) = fs::read_dir(&base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check if directory is older than 24 hours
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = now.duration_since(modified) {
                            if elapsed > one_day {
                                // Try to remove old directory, ignore errors
                                let _ = fs::remove_dir_all(&path);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn copy_jar_from_temp_dir_to_dest(tmp_jar_name: &String, final_path: &String) {
    fs::copy(get_or_create_limonium_dir().join(&tmp_jar_name), &final_path)
        .expect("Failed copying jar from temp directory to final path");
}

pub fn random_file_name(file_extension: &String) -> String {
    let mut tmp_jar_name = String::from("limonium-");
    tmp_jar_name.push_str(&Uuid::new_v4().to_string());
    tmp_jar_name.push_str(file_extension);
    return tmp_jar_name;
}

pub(crate) fn find_jar_files(dir: &Path, jar_pattern: &Regex) -> Vec<PathBuf> {
    let mut jar_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "jar"
                            && jar_pattern.is_match(path.file_name().unwrap().to_str().unwrap())
                        {
                            jar_files.push(path.clone());
                        }
                    }
                } else if path.is_dir() {
                    jar_files.extend(find_jar_files(&path, jar_pattern));
                }
            }
        }
    }

    jar_files
}
