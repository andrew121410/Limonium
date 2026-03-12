use std::fs;
use std::io::{Error, ErrorKind, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::{clap_utils, ensurer};
use chrono::{NaiveDate, Utc};
use colored::Colorize;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use regex::Regex;

#[derive(PartialEq)]
pub enum BackupFormat {
    TarGz,
    TarZst,
    Zip,
}

pub struct BackupResult {
    pub file_name: String,
    pub file_path: PathBuf,
    pub sha256_hash: String,
}

pub struct Backup {
    name: String,
    directory_to_backup: String,
    backup_directory: PathBuf,
    backup_format: BackupFormat,
    exclude: Option<String>,
    compression_level: Option<i64>,
}

impl Backup {
    pub fn new(name: String, directory_to_backup: String, backup_directory: PathBuf, backup_format: BackupFormat, exclude: Option<String>, compression_level: Option<i64>) -> Self {
        Backup {
            name,
            directory_to_backup,
            backup_directory,
            backup_format,
            exclude,
            compression_level,
        }
    }

    pub fn backup(&self) -> Result<BackupResult, Error> {
        let timestamp = chrono::Local::now().format("%-m-%-d-%Y");

        // The extension of the backup archive
        let extension = match self.backup_format {
            BackupFormat::TarGz => "tar.gz",
            BackupFormat::TarZst => "tar.zst",
            BackupFormat::Zip => "zip",
        };

        // Check if the compression format is installed
        match self.backup_format {
            BackupFormat::TarGz => {
                ensurer::Ensurer::ensure_programs(&[ensurer::Program::Tar, ensurer::Program::Gzip]);
            }
            BackupFormat::TarZst => {
                ensurer::Ensurer::ensure_programs(&[ensurer::Program::Tar, ensurer::Program::Zstd]);
            }
            BackupFormat::Zip => {
                ensurer::Ensurer::ensure_programs(&[ensurer::Program::Zip]);
            }
        }

        // Create the backup directory if it does not exist
        if !self.backup_directory.exists() {
            fs::create_dir_all(&self.backup_directory)?;
            println!("The backup directory did not exist, so it was created at {}", &self.backup_directory.display());
        }

        // Create a hidden temporary directory in the backup directory
        let our_tmp_directory = self.backup_directory.join(".lmtmp");

        // Create the temporary directory if it does not exist
        if !our_tmp_directory.exists() {
            fs::create_dir_all(&our_tmp_directory)?;
        } else {
            println!("{}", format!("Did you force quit the program last time?").red());
            println!("{}", format!("Please let me know if you did NOT force close the program last time").red());
            println!();
            // Error if the temporary directory already exists say where it is
            return Err(Error::new(ErrorKind::Other, format!("The temporary directory already exists at {}. Please delete it and try again.", our_tmp_directory.display())));
        }

        let backup_path = our_tmp_directory.join(format!("{}-{}.{}", &self.name, timestamp, extension));
        let hash_path = our_tmp_directory.join(format!("{}-{}_hash.txt", &self.name, timestamp));

        println!("{}", format!("Please wait while the backup is being created...").yellow());

        // Progress bar
        let bar = ProgressBar::new_spinner();
        let style = ProgressStyle::default_spinner()
            .tick_strings(&["-", "\\", "|", "/"])
            // Use ANSI escape code `\x1b[31m` for red, and `\x1b[0m` to reset the color
            .template("{spinner} \x1b[31m{elapsed_precise}\x1b[0m")
            .unwrap_or_else(|e| {
                eprintln!("Template error: {}", e);
                ProgressStyle::default_spinner()
            });
        bar.set_style(style);
        let start = Instant::now();
        bar.enable_steady_tick(Duration::from_millis(100));

        // Create compressed tar or zip archive of the Minecraft server files
        let mut cmd = Command::new("tar");
        match self.backup_format {
            BackupFormat::TarGz | BackupFormat::TarZst => {
                // You may exclude files and folders by splitting them with a : (colon)
                // Example: "logs:plugins/dynmap"
                if let Some(exclude) = &self.exclude {
                    let exclude = exclude.split(":");

                    for exclude in exclude {
                        cmd.arg(format!("--exclude={}", exclude));
                    }
                }

                let i_override = clap_utils::clap_get_one_or_fallback(&"I".to_string(), &"NONE".to_string());
                if self.backup_format == BackupFormat::TarZst {
                    // If we have a compression level, use it
                    if self.compression_level.is_some() && i_override.eq("NONE") {
                        let compression_level = self.compression_level.unwrap();

                        // Check if the compression level is between 1 and 22
                        if compression_level < 1 || compression_level > 22 {
                            return Err(Error::new(ErrorKind::Other, "The compression level must be between 1 and 22"));
                        }

                        // Compression level 20 to 22 uses --ultra
                        if compression_level >= 20 {
                            cmd.args(&["-I", &format!("zstd --ultra -{}", compression_level)]);
                        } else {
                            cmd.args(&["-I", &format!("zstd -{}", compression_level)]);
                        }
                    } else if !i_override.eq("NONE") {
                        if self.compression_level.is_some() {
                            return Err(Error::new(ErrorKind::Other, "The compression level flag (--level) and the override flag (-I) cannot be used at the same time. Please use one or the other."));
                        }

                        cmd.args(&["-I", &format!("{}", i_override)]);
                    } else { // If we don't have a compression level, use the default
                        cmd.arg("--zstd");
                    }
                    cmd.arg("-cf"); // c = create, f = file
                } else { // GZip
                    // If we have a compression level, use it
                    if self.compression_level.is_some() && i_override.eq("NONE") {
                        let compression_level = self.compression_level.unwrap();

                        // Check if the compression level is between 1 and 9
                        if compression_level < 1 || compression_level > 9 {
                            return Err(Error::new(ErrorKind::Other, "The compression level must be between 1 and 9"));
                        }

                        cmd.args(&["-I", &format!("gzip -{}", compression_level), "-cf"]);
                    } else if !i_override.eq("NONE") {
                        if self.compression_level.is_some() {
                            return Err(Error::new(ErrorKind::Other, "The compression level flag (--level) and the override flag (-I) cannot be used at the same time. Please use one or the other."));
                        }

                        cmd.args(&["-I", &format!("{}", i_override), "-cf"]);
                    } else { // If we don't have a compression level, use the default
                        cmd.arg("-czf"); // c = create, z = gzip, f = file
                    }
                }
                cmd.arg(&backup_path);

                // You may backup multiple folders by splitting them with a : (colon)
                // Example: "world:world_nether:world_the_end"
                let folders_to_backup = self.directory_to_backup.split(":");

                for folder in folders_to_backup {
                    cmd.arg(&folder);
                }
            }
            BackupFormat::Zip => {
                cmd = Command::new("zip");

                // If we have a compression level, use it
                if self.compression_level.is_some() {
                    let compression_level = self.compression_level.unwrap();

                    // Check if the compression level is between 1 and 9
                    if compression_level < 1 || compression_level > 9 {
                        return Err(Error::new(ErrorKind::Other, "The compression level must be between 1 and 9"));
                    }

                    cmd.arg(format!("-{}", compression_level));
                }

                cmd.arg("-r").arg(&backup_path);

                // You may backup multiple folders by splitting them with a : (colon)
                // Example: "world:world_nether:world_the_end"
                let folders_to_backup = self.directory_to_backup.split(":");

                for folder in folders_to_backup {
                    cmd.arg(&folder);
                }

                // You may exclude files and folders by splitting them with a : (colon)
                // Example: "logs:plugins/dynmap"
                if let Some(exclude) = &self.exclude {
                    let exclude = exclude.split(":");

                    for exclude in exclude {
                        cmd.arg(format!("-x {}", exclude));
                    }
                }
            }
        };

        let verbose = clap_utils::clap_get_flag_or_false(&"verbose".to_string());
        // Verbose before
        if verbose { // Capture the output of the backup command
            cmd.stdout(Stdio::piped());
        }

        let cmd_output = cmd.output()?;
        if !cmd_output.status.success() {
            return Err(Error::new(ErrorKind::Other, format!("Failed to create backup archive of Minecraft server files: {}", String::from_utf8_lossy(&cmd_output.stderr))));
        }

        // Verbose after
        if verbose {
            // Print the output of the backup command
            println!("{}", format!("Backup command output:").green());
            println!("{}", String::from_utf8_lossy(&cmd_output.stdout));

            // Print the backup command
            println!("{} {}", format!("Backup command:").green(), format!("{:?}", cmd).bright_yellow());
        }

        // Compute the sha256 hash of the backup archive
        let mut hash_cmd = Command::new("sha256sum");
        hash_cmd.arg(&backup_path);
        let hash_output = hash_cmd.output()?;
        if !hash_output.status.success() {
            return Err(Error::new(ErrorKind::Other, "Failed to compute hash of backup archive"));
        }

        // Write the hash to a file in the backup directory
        let mut hash_file = fs::File::create(&hash_path)?;
        hash_file.write_all(&hash_output.stdout)?;

        let how_many_backups_of_today_date = self.get_how_many_backups_of_today_date()?;
        let combined_backup_path = self.backup_directory.join(format!("{}-{}-{}-bundle.{}", &self.name, timestamp, how_many_backups_of_today_date, extension));

        // This should never happen, but just in case
        if combined_backup_path.exists() {
            return Err(Error::new(ErrorKind::Other, format!("The combined backup archive already exists at {}. This shouldn't have happened", combined_backup_path.display())));
        }

        // Create the combined backup archive with the backup archive and hash file, and it will be placed in the backup directory
        cmd = Command::new("tar");
        match self.backup_format {
            BackupFormat::TarGz => {
                cmd.arg("-czf").arg(&combined_backup_path);
                cmd.arg("-C").arg(&our_tmp_directory);
                cmd.arg(&backup_path.file_name().unwrap());
                cmd.arg(&hash_path.file_name().unwrap());
            }
            BackupFormat::TarZst => {
                cmd.arg("--zstd").arg("-cf").arg(&combined_backup_path);
                cmd.arg("-C").arg(&our_tmp_directory);
                cmd.arg(&backup_path.file_name().unwrap());
                cmd.arg(&hash_path.file_name().unwrap());
            }
            BackupFormat::Zip => {
                cmd = Command::new("zip");
                cmd.current_dir(&our_tmp_directory);

                cmd.arg("-r").arg(&combined_backup_path);
                cmd.arg(&backup_path.file_name().unwrap());
                cmd.arg(&hash_path.file_name().unwrap());
            }
        }

        let cmd_output = cmd.output()?;
        if !cmd_output.status.success() {
            return Err(Error::new(ErrorKind::Other, format!("Failed to create combined backup archive of Minecraft server files and hash file: {}", String::from_utf8_lossy(&cmd_output.stderr))));
        }

        // Delete the temporary backup archive and hash file in the temporary directory
        fs::remove_file(&backup_path).expect("Failed to delete temporary backup archive");
        fs::remove_file(&hash_path).expect("Failed to delete temporary hash file");

        // Delete the temporary directory
        fs::remove_dir_all(&our_tmp_directory).expect("Failed to delete temporary directory");

        // Create hash of combined backup archive
        let mut hash_cmd = Command::new("sha256sum");
        hash_cmd.arg(&combined_backup_path);
        let hash_output = hash_cmd.output()?;
        if !hash_output.status.success() {
            return Err(Error::new(ErrorKind::Other, "Failed to compute hash of combined backup archive"));
        }

        let combined_backup_hash = String::from_utf8_lossy(&hash_output.stdout).split(" ").collect::<Vec<&str>>()[0].to_string();
        let backup_result = BackupResult {
            file_name: combined_backup_path.file_name().unwrap().to_str().unwrap().to_string(),
            file_path: combined_backup_path,
            sha256_hash: combined_backup_hash,
        };

        // Finish the progress bar
        let duration = HumanDuration(start.elapsed());
        let progress_bar_ending_message = format!("Task completed in {}", duration);
        bar.finish_with_message(progress_bar_ending_message);

        // Display the result of the backup
        println!("{} {}", format!("Local backup finished!").bright_green(), format!("Details below:").yellow());
        println!("{} {}", format!("Backup file:").green(), format!("{}", &backup_result.file_name).bright_yellow());
        println!("{}", format!("Backup (sha256) hash: {}", &backup_result.sha256_hash).green());
        // Size show in MB, but if higher than 1GB show in GB
        if backup_result.file_path.metadata().unwrap().len() > 1073741824 {
            let size = backup_result.file_path.metadata().unwrap().len() as f64 / 1073741824.0;
            println!("{} {} {}", format!("Backup size:").green(), format!("{:.2}", size).bright_yellow(), format!("GB").bright_cyan());
        } else {
            let size = backup_result.file_path.metadata().unwrap().len() as f64 / 1048576.0;
            println!("{} {} {}", format!("Backup size:").green(), format!("{:.2}", size).bright_yellow(), format!("MB").bright_cyan());
        }

        Ok(backup_result)
    }


    pub fn local_delete_after_time(&self, input: &String, always_keep: Option<usize>) {
        // Deletes backups after a certain amount of time, optionally keeping at least `always_keep` files.

        let backup_directory = self.backup_directory.clone();

        // Parse the input duration
        let (amount, unit) = input.split_at(input.len() - 1);
        let amount: i64 = match amount.parse() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Invalid amount provided.");
                return;
            }
        };

        // Get the current date
        let current_date = Utc::now().naive_utc().date();

        // Collect all valid backups
        let mut backups = fs::read_dir(&backup_directory)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name()?.to_str()?;
                    let date = extract_date_from_file_name(&file_name.to_string());
                    if !date.is_empty() && file_name.starts_with(&self.name) {
                        let date_chrono = NaiveDate::parse_from_str(&date, "%-m-%-d-%Y").ok()?;
                        let days_difference = (current_date - date_chrono).num_days();
                        Some((path, days_difference))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Sort backups by age, oldest first
        backups.sort_by_key(|&(_, days_difference)| days_difference);

        for (path, days_difference) in &backups {
            // Check if `always_keep` is set and if deleting this file would fall below the threshold
            if let Some(min_count) = always_keep {
                if backups.len() <= min_count {
                    println!("Keeping {} as it falls within the always-keep threshold.", path.display());
                    break;
                }
            }

            // Determine if the file should be deleted based on the input duration
            let should_delete = match unit {
                "m" => *days_difference > amount * 30,
                "w" => *days_difference > amount * 7,
                "d" => *days_difference > amount,
                _ => {
                    eprintln!("Invalid time unit: {}", unit);
                    false
                }
            };

            // Delete if it meets deletion criteria
            if should_delete {
                if let Err(e) = fs::remove_file(path) {
                    eprintln!("Error deleting file: {}", e);
                } else {
                    println!("Deleted file {} as it was too old.", path.display());
                }
            }
        }
    }


    /*
    Determine how many backups of today's date exist
    If there are none backups, of today's date, the return will be 1
    If there is 1 backup, of today's date, the return will be 2 so on and so forth
     */
    fn get_how_many_backups_of_today_date(&self) -> Result<i64, Error> {
        let timestamp = chrono::Local::now().format("%-m-%-d-%Y");
        let mut count = 1;

        for entry in fs::read_dir(&self.backup_directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if file_name.starts_with(&format!("{}-{}", self.name, timestamp)) {
                    count += 1;
                }
            }
        }
        Ok(count)
    }
}

pub(crate) fn extract_date_from_file_name(file_name: &String) -> String {
    // Define a regex pattern for capturing the date part
    let date_pattern = Regex::new(r"(\d{1,2}-\d{1,2}-\d{4})").unwrap();

    // Find the first match in the file name
    if let Some(captures) = date_pattern.find(file_name) {
        return captures.as_str().to_string();
    }

    // Default value if no match is found
    String::new()
}

#[cfg(test)]
mod backup_testing {
    use std::fs::File;

    use chrono::Duration;

    use crate::backup::extract_date_from_file_name;

    use super::*;

    #[test]
    fn test_extract_date_from_file_name() {
        assert_eq!(extract_date_from_file_name(&"hub-everything-10-19-2023-1-bundle.tar.zst".to_string()), "10-19-2023");
        assert_eq!(extract_date_from_file_name(&"hub-11-15-2023-1-bundle.tar.zst".to_string()), "11-15-2023");
        assert_eq!(extract_date_from_file_name(&"hub-11-15-2023-2-bundle.tar.zst".to_string()), "11-15-2023");
        assert_eq!(extract_date_from_file_name(&"hub-10-30-2023-1-bundle.tar.zst".to_string()), "10-30-2023");
        assert_eq!(extract_date_from_file_name(&"testing-9-29-2023-1-bundle.tar.zst".to_string()), "9-29-2023");
        assert_eq!(extract_date_from_file_name(&"testing-9-29-2023-1-bundle.zip".to_string()), "9-29-2023");
        assert_eq!(extract_date_from_file_name(&"testing-9-29-2023-2-bundle.zip".to_string()), "9-29-2023");
        assert_eq!(extract_date_from_file_name(&"testing-everything-9-29-2023-1-bundle.zip".to_string()), "9-29-2023");
        assert_eq!(extract_date_from_file_name(&"hub-11-15-2023-1-bundle.tar.zst".to_string()), "11-15-2023");
        assert_eq!(extract_date_from_file_name(&"can-have-infinite-dashes-right-here-11-4-2023-1-bundle.tar.zst".to_string()), "11-4-2023");
        assert_eq!(extract_date_from_file_name(&"can-have-infinite-dashes-right-here-11-4-2023-2-bundle.tar.zst".to_string()), "11-4-2023");
        assert_eq!(extract_date_from_file_name(&"can-have-infinite-dashes-right-here-11-4-2023-1-bundle.zip".to_string()), "11-4-2023");
    }

    #[test]
    fn test_local_delete_after_time() {
        // Create a temporary directory for testing
        let temp_dir_dir_to_backup = tempdir::TempDir::new("directory-to-backup").expect("Failed to create temp dir");
        let temp_dir_backup_directory = tempdir::TempDir::new("backup-directory").expect("Failed to create temp dir");

        // Define a backup object with a sample configuration
        let backup = Backup {
            name: "testing-backup".to_string(),
            directory_to_backup: temp_dir_dir_to_backup.path().to_string_lossy().to_string(),
            backup_directory: temp_dir_backup_directory.path().to_path_buf(),
            backup_format: BackupFormat::TarGz,
            exclude: None,
            compression_level: None,
        };

        // Create a backup file with a date that should be deleted based on the provided input
        let old_backup_date = Utc::now().naive_utc().date() - Duration::days(10); // Example: 10 days old
        let old_backup_file_name = format!("{}-{}.tar.gz", backup.name, old_backup_date.format("%-m-%-d-%Y"));
        let old_backup_file_path = backup.backup_directory.join(&old_backup_file_name);
        File::create(&old_backup_file_path).expect("Failed to create old backup file");

        // Create a backup file with a recent date that should not be deleted
        let recent_backup_date = Utc::now().naive_utc().date() - Duration::days(2); // Example: 2 days old
        let recent_backup_file_name = format!("{}-{}.tar.gz", backup.name, recent_backup_date.format("%-m-%-d-%Y"));
        let recent_backup_file_path = backup.backup_directory.join(&recent_backup_file_name);
        File::create(&recent_backup_file_path).expect("Failed to create recent backup file");

        // Define the input for deletion (e.g., 7d for 7 days)
        let deletion_input = "7d".to_string();

        // Call the local_delete_after_time function
        backup.local_delete_after_time(&deletion_input, None);

        // Check if the old backup file is deleted
        assert!(!old_backup_file_path.exists(), "Old backup file should be deleted");

        // Check if the recent backup file is not deleted
        assert!(recent_backup_file_path.exists(), "Recent backup file should not be deleted");
    }

    #[test]
    fn test_local_delete_after_time_with_always_keep() {
        // Create a temporary directory for testing
        let temp_dir_dir_to_backup = tempdir::TempDir::new("directory-to-backup").expect("Failed to create temp dir");
        let temp_dir_backup_directory = tempdir::TempDir::new("backup-directory").expect("Failed to create temp dir");

        // Define a backup object with a sample configuration
        let backup = Backup {
            name: "testing-backup".to_string(),
            directory_to_backup: temp_dir_dir_to_backup.path().to_string_lossy().to_string(),
            backup_directory: temp_dir_backup_directory.path().to_path_buf(),
            backup_format: BackupFormat::TarGz,
            exclude: None,
            compression_level: None,
        };

        // Create a backup file with a date that should be deleted based on the provided input
        let old_backup_date = Utc::now().naive_utc().date() - Duration::days(10); // Example: 10 days old
        let old_backup_file_name = format!("{}-{}.tar.gz", backup.name, old_backup_date.format("%-m-%-d-%Y"));
        let old_backup_file_path = backup.backup_directory.join(&old_backup_file_name);
        File::create(&old_backup_file_path).expect("Failed to create old backup file");

        // Create a backup file with a recent date that should not be deleted
        let recent_backup_date = Utc::now().naive_utc().date() - Duration::days(2); // Example: 2 days old
        let recent_backup_file_name = format!("{}-{}.tar.gz", backup.name, recent_backup_date.format("%-m-%-d-%Y"));
        let recent_backup_file_path = backup.backup_directory.join(&recent_backup_file_name);
        File::create(&recent_backup_file_path).expect("Failed to create recent backup file");

        // Define the input for deletion (e.g., 7d for 7 days)
        let deletion_input = "7d".to_string();

        // Call the local_delete_after_time function with always_keep set to 2
        backup.local_delete_after_time(&deletion_input, Some(2));

        // Check if the old backup file is not deleted because of the always_keep threshold
        assert!(old_backup_file_path.exists(), "Old backup file should not be deleted due to always_keep");

        // Check if the recent backup file is also not deleted
        assert!(recent_backup_file_path.exists(), "Recent backup file should not be deleted");
    }
}