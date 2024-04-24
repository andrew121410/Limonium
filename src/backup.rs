use std::fs;
use std::io::{BufRead, Error, ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use chrono::{NaiveDate, Utc};
use colored::Colorize;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use openssh::{Session, Stdio};
use openssh_sftp_client::Sftp;
use regex::Regex;

use crate::controllers;

#[derive(PartialEq)]
pub enum BackupFormat {
    TarGz,
    TarZst,
    Zip,
}

pub struct Backup_Result {
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

    pub fn backup(&self) -> Result<Backup_Result, Error> {
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
                if !does_tar_command_exist() {
                    return Err(Error::new(ErrorKind::Other, "The tar command does not exist. Please install it and try again."));
                }
            }
            BackupFormat::TarZst => {
                if !does_zstd_command_exist() {
                    return Err(Error::new(ErrorKind::Other, "The zstd command does not exist. Please install it and try again."));
                }
            }
            BackupFormat::Zip => {
                if !does_zip_command_exist() {
                    return Err(Error::new(ErrorKind::Other, "The zip command does not exist. Please install it and try again."));
                }
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

                let i_override = controllers::clap_get_one_or_fallback(&"I".to_string(), &"NONE".to_string());
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

        let verbose = controllers::clap_get_flag_or_fallback(&"verbose".to_string());
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
        let backup_result = Backup_Result {
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

    pub async fn upload_sftp(&self, user: String, host: String, port: Option<u16>, key_file: Option<&Path>, path: &PathBuf, file_name: String, remote_dir: String, local_hash: String) -> Result<(), Error> {
        let real_session: Session = sftp_login(user, host, port, key_file).await?;

        let mut child = real_session
            .subsystem("sftp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .await
            .unwrap();

        let sftp = Sftp::new(
            child.stdin().take().unwrap(),
            child.stdout().take().unwrap(),
            Default::default(), )
            .await
            .unwrap();

        let mut fs = sftp.fs();

        let remote_create_dir_result = fs.create_dir(Path::new(&remote_dir)).await;
        if remote_create_dir_result.is_err() {
            // Ignore error the directory probably already exists
        }

        let remote_file_result = sftp.create(Path::new(&remote_dir).join(&file_name)).await;

        if remote_file_result.is_err() {
            println!("{}", format!("Failed to create remote file: {}", remote_file_result.err().unwrap()).red());
            return Err(Error::new(ErrorKind::Other, "Failed to create remote file"));
        }

        let mut remote_file = remote_file_result.unwrap();
        let mut local_file = fs::File::open(path).unwrap();

        // Progress bar in mb
        let progress_bar = ProgressBar::new(local_file.metadata().unwrap().len());
        progress_bar.set_style(ProgressStyle::default_bar()
            .template("{msg:>12.cyan.bold} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap());
        progress_bar.set_message("Uploading");

        // Split the file into chunks to upload
        const CHUNK_SIZE: usize = 1024 * 1024 * 3; // 3 MB
        let mut buffer = vec![0; CHUNK_SIZE];
        loop {
            // Read a chunk from the local file
            let bytes_read = local_file.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            // Write the chunk to the remote file
            remote_file.write_all(&buffer[..bytes_read]).await.unwrap();

            // Update the progress bar
            progress_bar.inc(bytes_read as u64);

            buffer = vec![0; CHUNK_SIZE]; // Reallocate the buffer (why doesn't buffer.clear() work?)
        }

        progress_bar.finish_and_clear();

        // Verify that the file was uploaded correctly (check the hash)
        let mut command = real_session.command("sha256sum".to_string());
        command.arg(format!("{}/{}", remote_dir, file_name));
        let output = command.output().await.unwrap();
        let output_string = String::from_utf8_lossy(&output.stdout);
        let remote_hash = output_string.split(" ").collect::<Vec<&str>>()[0].to_string();

        if remote_hash != local_hash {
            println!("{}", format!("Failed to upload backup archive to SFTP server: The hash of the local file ({}) does not match the hash of the remote file ({})", local_hash, remote_hash).red());
            return Err(Error::new(ErrorKind::Other, "Local and remote hash do not match"));
        }

        println!("{}", format!("Hash of local file matches hash of remote file").green());
        println!("{}", format!("Successfully uploaded backup archive to SFTP server").green());
        Ok(())
    }

    pub fn local_delete_after_time(&self, input: &String) {
        // Deletes backups after a certain amount of time
        // Example input: 1m = 1 month, 1w = 1 week, 1d = 1 day

        let backup_directory = self.backup_directory.clone();

        // Parse the input duration
        let (amount, unit) = input.split_at(input.len() - 1);
        let amount: i64 = amount.parse().unwrap_or(0);

        // Get the current date
        let current_date = Utc::now().naive_utc().date();

        // For each file in the backup directory
        for entry in fs::read_dir(&backup_directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let date = extract_date_from_file_name(&file_name.to_string());

                // If the file name does not contain a date or does not start with the backup name, skip it
                if date.is_empty() || !file_name.starts_with(&self.name) {
                    continue;
                }

                let date_chrono = NaiveDate::parse_from_str(&date, "%-m-%-d-%Y").unwrap();

                // Calculate the difference in days between the current date and the backup date
                let days_difference = (current_date - date_chrono).num_days();

                // Delete if it's older than the input duration
                match unit {
                    "m" => {
                        if days_difference > amount * 30 {
                            // Delete the file
                            fs::remove_file(&path).unwrap_or_else(|e| {
                                eprintln!("Error deleting file: {}", e);
                            });
                            println!("{}", format!("(delete-after-time) Deleted file {} in the backup directory it was too OLD", file_name).yellow());
                        }
                    }
                    "w" => {
                        if days_difference > amount * 7 {
                            // Delete the file
                            fs::remove_file(&path).unwrap_or_else(|e| {
                                eprintln!("Error deleting file: {}", e);
                            });
                            println!("{}", format!("(delete-after-time) Deleted file {} in the backup directory it was too OLD", file_name).yellow());
                        }
                    }
                    "d" => {
                        if days_difference > amount {
                            // Delete the file
                            fs::remove_file(&path).unwrap_or_else(|e| {
                                eprintln!("Error deleting file: {}", e);
                            });
                            println!("{}", format!("(delete-after-time) Deleted file {} in the backup directory it was too OLD", file_name).yellow());
                        }
                    }
                    _ => {
                        eprintln!("Invalid time unit: {}", unit);
                    }
                }
            }
        }
    }

    pub async fn sftp_delete_after_time(&self, input: &String, user: String, host: String, port: Option<u16>, key_file: Option<&Path>, remote_dir: String) {
        // Deletes backups after a certain amount of time in the SFTP server
        // Example input: 1m = 1 month, 1w = 1 week, 1d = 1 day

        let sftp_result = sftp_login(user, host, port, key_file).await.unwrap();

        // Parse the input duration
        let (amount, unit) = input.split_at(input.len() - 1);
        let amount: i64 = amount.parse().unwrap_or(0);

        // Get the current date
        let current_date = Utc::now().naive_utc().date();

        // For each file in the backup directory
        let file_names = list_files_on_sftp(&sftp_result, &remote_dir).await.unwrap();

        for file_name in file_names {
            let date = extract_date_from_file_name(&file_name.to_string());

            // If the file name does not contain a date or does not start with the backup name, skip it
            if date.is_empty() || !file_name.starts_with(&self.name) {
                continue;
            }

            let date_chrono = NaiveDate::parse_from_str(&date, "%-m-%-d-%Y").unwrap();

            // Calculate the difference in days between the current date and the backup date
            let days_difference = (current_date - date_chrono).num_days();

            // Delete if it's older than the input duration
            match unit {
                "m" => {
                    if days_difference > amount * 30 {
                        // Delete the file
                        delete_file_on_sftp(&sftp_result, &file_name, &remote_dir).await.unwrap_or_else(|e| {
                            eprintln!("Error deleting file: {}", e);
                        });
                    }
                }
                "w" => {
                    if days_difference > amount * 7 {
                        // Delete the file
                        delete_file_on_sftp(&sftp_result, &file_name, &remote_dir).await.unwrap_or_else(|e| {
                            eprintln!("Error deleting file: {}", e);
                        });
                    }
                }
                "d" => {
                    if days_difference > amount {
                        // Delete the file
                        delete_file_on_sftp(&sftp_result, &file_name, &remote_dir).await.unwrap_or_else(|e| {
                            eprintln!("Error deleting file: {}", e);
                        });
                    }
                }
                _ => {
                    eprintln!("Invalid time unit: {}", unit);
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

async fn sftp_login(user: String, host: String, port: Option<u16>, key_file: Option<&Path>) -> Result<Session, Error> {
    let mut session_builder = openssh::SessionBuilder::default();

    if key_file.is_some() {
        session_builder.keyfile(key_file.unwrap());
        println!("Using key file: {}", key_file.unwrap().display());

        // Check if right permissions are set on the key file
        let metadata = fs::metadata(key_file.unwrap()).unwrap();
        let permissions = metadata.permissions();
        if permissions.mode() != 33152 { // chmod 600
            println!("{}", format!("The key file must have the permissions 600 (rw-------). Please run \"chmod 600 {}\" to set the correct permissions.", key_file.unwrap().display()).red());
            return Err(Error::new(ErrorKind::Other, "Wrong permissions on key file"));
        }
    }

    session_builder.user(user);

    if port.is_some() {
        session_builder.port(port.unwrap());
    }

    let session_result = session_builder.connect(&host).await;
    if session_result.is_err() {
        println!("{}", format!("Failed to connect to {}: {}", host, session_result.err().unwrap()).red());
        return Err(Error::new(ErrorKind::Other, "Failed to connect to host"));
    }

    Ok(session_result.unwrap())
}

async fn list_files_on_sftp(session: &Session, remote_dir: &String) -> Result<Vec<String>, Error> {
    let mut file_names: Vec<String> = Vec::new();

    // Just use ls command
    let mut command = session.command("ls".to_string());
    command.arg(remote_dir);

    // Execute the command and capture the output
    let output = command.output().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Check if the command was successful
    if output.status.success() {
        // Convert the output bytes to a string
        let output_str = String::from_utf8(output.stdout).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Split the output by newline characters and collect the file names
        file_names = output_str.split('\n').map(|s| s.to_string()).collect();
    } else {
        // Return an error if the command failed
        return Err(Error::new(
            ErrorKind::Other,
            format!("ls command failed: {}", output.status),
        ));
    }

    // Return the file names
    Ok(file_names)
}

async fn delete_file_on_sftp(session: &Session, file_name: &String, remote_dir: &String) -> Result<(), Error> {
    // Delete file using rm command
    let mut command = session.command("rm".to_string());
    command.arg(format!("{}/{}", remote_dir, file_name));
    let output = command.output().await.unwrap();
    let output_string = String::from_utf8_lossy(&output.stdout);

    if output_string.contains("No such file") {
        println!("{}", format!("The file {} does not exist on the SFTP server", file_name).red());
    }

    println!("{}", format!("(delete-after-time) Deleted file {} in the backup directory on the SFTP server it was too OLD", file_name).yellow());
    Ok(())
}

fn extract_date_from_file_name(file_name: &String) -> String {
    // Define a regex pattern for capturing the date part
    let date_pattern = Regex::new(r"(\d{1,2}-\d{1,2}-\d{4})").unwrap();

    // Find the first match in the file name
    if let Some(captures) = date_pattern.find(file_name) {
        return captures.as_str().to_string();
    }

    // Default value if no match is found
    String::new()
}

fn does_tar_command_exist() -> bool {
    let output = Command::new("tar").arg("--version").output();

    if output.is_err() {
        return false;
    }

    let the_output = output.unwrap();
    the_output.status.success()
}

fn does_zstd_command_exist() -> bool {
    let output = Command::new("zstd").arg("--version").output();

    if output.is_err() {
        return false;
    }

    let the_output = output.unwrap();
    the_output.status.success()
}

fn does_zip_command_exist() -> bool {
    let output = Command::new("zip").arg("--version").output();

    if output.is_err() {
        return false;
    }

    let the_output = output.unwrap();
    the_output.status.success()
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
        backup.local_delete_after_time(&deletion_input);

        // Check if the old backup file is deleted
        assert!(!old_backup_file_path.exists(), "Old backup file should be deleted");

        // Check if the recent backup file is not deleted
        assert!(recent_backup_file_path.exists(), "Recent backup file should not be deleted");
    }
}