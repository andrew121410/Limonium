use std::path::PathBuf;

pub struct DownloadedFile {
    pub real_file_name: Option<String>,
    pub temp_file_name: String,
    pub temp_file_path: PathBuf,
}

impl DownloadedFile {
    pub fn empty() -> Self {
        Self {
            real_file_name: None,
            temp_file_name: String::new(),
            temp_file_path: PathBuf::new(),
        }
    }
}