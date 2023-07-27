use std::path::PathBuf;

pub struct DownloadedJar {
    pub real_jar_name: Option<String>,
    pub temp_jar_name: String,
    pub temp_jar_path: PathBuf,
}

impl DownloadedJar {
    pub fn empty() -> Self {
        Self {
            real_jar_name: None,
            temp_jar_name: String::new(),
            temp_jar_path: PathBuf::new(),
        }
    }
}