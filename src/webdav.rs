use std::io::{Error, ErrorKind, Read};
use std::path::PathBuf;

use chrono::{NaiveDate, Utc};
use colored::Colorize;
use futures_util::stream;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::Client;

use crate::backup::extract_date_from_file_name;

pub struct WebDavClient {
    client: Client,
    base_url: String,
    username: String,
    password: String,
}

impl WebDavClient {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        let client = Client::new();
        // Remove trailing slash from base_url
        let base_url = base_url.trim_end_matches('/').to_string();
        WebDavClient { client, base_url, username, password }
    }

    pub async fn upload_file(&self, path: &PathBuf, file_name: &str) -> Result<(), Error> {
        // Ensure remote directory exists (MKCOL)
        self.create_directory().await.ok(); // Ignore error if already exists

        let file = std::fs::File::open(path)
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to open file: {}", e)))?;
        let file_size = file.metadata()
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to get file metadata: {}", e)))?.len();

        // Progress bar
        let progress_bar = ProgressBar::new(file_size);
        progress_bar.set_style(ProgressStyle::default_bar()
            .template("{msg:>12.cyan.bold} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap());
        progress_bar.set_message("Uploading");

        // Stream the file in chunks with progress tracking
        let pb_clone = progress_bar.clone();
        let body_stream = stream::unfold(file, move |mut file| {
            let pb = pb_clone.clone();
            async move {
                let mut buf = vec![0u8; 3 * 1024 * 1024]; // 3 MB chunks
                match file.read(&mut buf) {
                    Ok(0) => None,
                    Ok(n) => {
                        buf.truncate(n);
                        pb.inc(n as u64);
                        Some((Ok::<_, std::io::Error>(buf), file))
                    }
                    Err(e) => Some((Err(e), file))
                }
            }
        });

        let url = format!("{}/{}", self.base_url, file_name);
        let response = self.client.put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .body(reqwest::Body::wrap_stream(body_stream))
            .send()
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to upload file: {}", e)))?;

        progress_bar.finish_and_clear();

        let status = response.status();
        if !status.is_success() && status.as_u16() != 201 && status.as_u16() != 204 {
            return Err(Error::new(ErrorKind::Other, format!("WebDAV upload failed with status: {}", status)));
        }

        // Verify upload by checking file size with a HEAD request
        let head_response = self.client.head(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to verify upload: {}", e)))?;

        if let Some(content_length) = head_response.content_length() {
            if content_length != file_size {
                return Err(Error::new(ErrorKind::Other,
                    format!("Upload verification failed: local size ({}) != remote size ({})", file_size, content_length)));
            }
            println!("{}", "File size verified: local and remote sizes match".green());
        } else {
            println!("{}", "Warning: WebDAV server did not return Content-Length, skipping size verification".yellow());
        }

        println!("{}", "Successfully uploaded backup archive to WebDAV server".green());
        Ok(())
    }

    pub async fn delete_after_time(&self, backup_name: &str, input: &str) {
        // Deletes backups after a certain amount of time on the WebDAV server
        // Example input: 1m = 1 month, 1w = 1 week, 1d = 1 day

        // Parse the input duration
        let (amount, unit) = input.split_at(input.len() - 1);
        let amount: i64 = amount.parse().unwrap_or(0);

        let current_date = Utc::now().naive_utc().date();

        let file_names = match self.list_files().await {
            Ok(files) => files,
            Err(e) => {
                eprintln!("Failed to list files on WebDAV server: {}", e);
                return;
            }
        };

        for file_name in file_names {
            let date = extract_date_from_file_name(&file_name);

            if date.is_empty() || !file_name.starts_with(backup_name) {
                continue;
            }

            let date_chrono = match NaiveDate::parse_from_str(&date, "%-m-%-d-%Y") {
                Ok(d) => d,
                Err(_) => continue,
            };

            let days_difference = (current_date - date_chrono).num_days();

            let should_delete = match unit {
                "m" => days_difference > amount * 30,
                "w" => days_difference > amount * 7,
                "d" => days_difference > amount,
                _ => {
                    eprintln!("Invalid time unit: {}", unit);
                    false
                }
            };

            if should_delete {
                self.delete_file(&file_name).await.unwrap_or_else(|e| {
                    eprintln!("Error deleting file: {}", e);
                });
            }
        }
    }

    async fn create_directory(&self) -> Result<(), Error> {
        let response = self.client.request(
            reqwest::Method::from_bytes(b"MKCOL").unwrap(),
            &self.base_url,
        )
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to create directory: {}", e)))?;

        // 201 Created, 405 Method Not Allowed (already exists on some servers)
        let status = response.status();
        if !status.is_success() && status.as_u16() != 405 {
            return Err(Error::new(ErrorKind::Other, format!("MKCOL failed with status: {}", status)));
        }

        Ok(())
    }

    async fn list_files(&self) -> Result<Vec<String>, Error> {
        let response = self.client.request(
            reqwest::Method::from_bytes(b"PROPFIND").unwrap(),
            &self.base_url,
        )
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(r#"<?xml version="1.0" encoding="utf-8"?><D:propfind xmlns:D="DAV:"><D:prop><D:displayname/></D:prop></D:propfind>"#)
            .send()
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("PROPFIND failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() && status.as_u16() != 207 {
            return Err(Error::new(ErrorKind::Other, format!("PROPFIND failed with status: {}", status)));
        }

        let body = response.text().await
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to read PROPFIND response: {}", e)))?;

        // Parse href elements from the WebDAV XML response
        // Handles both <D:href> and <d:href> namespace prefixes, as well as <href> without prefix
        let href_pattern = Regex::new(r"<(?:[Dd]:)?href>([^<]+)</(?:[Dd]:)?href>").unwrap();
        let mut file_names = Vec::new();

        for cap in href_pattern.captures_iter(&body) {
            let href = &cap[1];
            let decoded = urldecode(href);
            // Get just the filename (last path segment)
            let name = decoded.trim_end_matches('/').rsplit('/').next().unwrap_or("").to_string();
            if !name.is_empty() {
                file_names.push(name);
            }
        }

        // The first entry is typically the directory itself, skip it
        if !file_names.is_empty() {
            file_names.remove(0);
        }

        Ok(file_names)
    }

    async fn delete_file(&self, file_name: &str) -> Result<(), Error> {
        let url = format!("{}/{}", self.base_url, file_name);
        let response = self.client.delete(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to delete file: {}", e)))?;

        let status = response.status();
        if !status.is_success() && status.as_u16() != 204 {
            println!("{}", format!("Failed to delete {} on WebDAV server: {}", file_name, status).red());
            return Err(Error::new(ErrorKind::Other, format!("DELETE failed with status: {}", status)));
        }

        println!("{}", format!("(delete-after-time) Deleted file {} on the WebDAV server, it was too OLD", file_name).yellow());
        Ok(())
    }
}

/// Simple percent-decoding for URL paths
fn urldecode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else {
            result.push(c);
        }
    }
    result
}

