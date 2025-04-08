use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use uuid::Uuid;

pub fn get_or_create_limonium_dir() -> PathBuf {
    let mut dir = env::temp_dir();
    dir.push("limonium");

    if !dir.exists() {
        fs::create_dir_all(&dir);
    }

    dir
}

pub fn delete_limonium_folder() -> std::io::Result<()> {
    let mut dir = env::temp_dir();
    dir.push("limonium");

    if dir.exists() {
        fs::remove_dir_all(&dir)?;
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