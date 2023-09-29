use std::fs;
use std::io::{Error, ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use openssh::Stdio;
use openssh_sftp_client::Sftp;

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
                if !self.does_tar_command_exist() {
                    return Err(Error::new(ErrorKind::Other, "The tar command does not exist. Please install it and try again."));
                }
            }
            BackupFormat::TarZst => {
                if !self.does_zstd_command_exist() {
                    return Err(Error::new(ErrorKind::Other, "The zstd command does not exist. Please install it and try again."));
                }
            }
            BackupFormat::Zip => {
                if !self.does_zip_command_exist() {
                    return Err(Error::new(ErrorKind::Other, "The zip command does not exist. Please install it and try again."));
                }
            }
        }

        // Create the backup directory if it does not exist
        if !self.backup_directory.exists() {
            fs::create_dir_all(&self.backup_directory)?;
            println!("The backup directory did not exist, so it was created at {}", &self.backup_directory.display());
        }

        let backup_path = self.backup_directory.join(format!("{}-{}.{}", &self.name, timestamp, extension));
        let hash_path = self.backup_directory.join(format!("{}-{}_hash.txt", &self.name, timestamp));

        println!("{}", format!("Please wait while the backup is being created...").yellow());

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

                if self.backup_format == BackupFormat::TarZst {
                    // If we have a compression level, use it
                    if self.compression_level.is_some() {
                        let compression_level = self.compression_level.unwrap();

                        // Check if the compression level is between 1 and 22
                        if compression_level < 1 || compression_level > 22 {
                            return Err(Error::new(ErrorKind::Other, "The compression level must be between 1 and 22"));
                        }

                        // Compression level 20 to 22 uses --ultra
                        if compression_level >= 20 {
                            cmd.arg(&format!("-I \"zstd --ultra -{}\"", compression_level));
                        } else {
                            cmd.arg(&format!("-I \"zstd -{}\"", compression_level));
                        }
                    } else { // If we don't have a compression level, use the default
                        cmd.arg("--zstd");
                    }
                    cmd.arg("-cf"); // c = create, f = file
                } else {
                    // If we have a compression level, use it
                    if self.compression_level.is_some() {
                        let compression_level = self.compression_level.unwrap();

                        // Check if the compression level is between 1 and 9
                        if compression_level < 1 || compression_level > 9 {
                            return Err(Error::new(ErrorKind::Other, "The compression level must be between 1 and 9"));
                        }

                        cmd.arg("-I");
                        cmd.arg(format!("gzip -{}", compression_level));
                        cmd.arg("-cf");
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

        // Testing
        println!("{:?}", cmd);

        let cmd_output = cmd.output()?;
        if !cmd_output.status.success() {
            return Err(Error::new(ErrorKind::Other, format!("Failed to create backup archive of Minecraft server files: {}", String::from_utf8_lossy(&cmd_output.stderr))));
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

        // Combine the backup archive and the hash into a single archive
        let how_many_backups_of_today_date = self.get_how_many_backups_of_today_date()?;
        let combined_backup_path = self.backup_directory.join(format!("{}-{}-{}-bundle.{}", &self.name, timestamp, how_many_backups_of_today_date, extension));

        cmd = Command::new("tar");
        match self.backup_format {
            BackupFormat::TarGz => {
                cmd.arg("-czf").arg(&combined_backup_path);
                cmd.arg("-C").arg(&self.backup_directory);
                cmd.arg(&backup_path.file_name().unwrap());
                cmd.arg(&hash_path.file_name().unwrap());
            }
            BackupFormat::TarZst => {
                cmd.arg("--zstd").arg("-cf").arg(&combined_backup_path);
                cmd.arg("-C").arg(&self.backup_directory);
                cmd.arg(&backup_path.file_name().unwrap());
                cmd.arg(&hash_path.file_name().unwrap());
            }
            BackupFormat::Zip => {
                cmd = Command::new("zip");
                cmd.current_dir(&self.backup_directory);

                cmd.arg("-r").arg(&combined_backup_path);
                cmd.arg(&backup_path.file_name().unwrap());
                cmd.arg(&hash_path.file_name().unwrap());
            }
        }

        let cmd_output = cmd.output()?;
        if !cmd_output.status.success() {
            return Err(Error::new(ErrorKind::Other, format!("Failed to create combined backup archive of Minecraft server files and hash file: {}", String::from_utf8_lossy(&cmd_output.stderr))));
        }

        // Delete the temporary backup archive and hash file
        fs::remove_file(&backup_path).expect("Failed to delete temporary backup archive");
        fs::remove_file(&hash_path).expect("Failed to delete temporary hash file");

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

        let real_session = session_result.unwrap();

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

    fn get_how_many_backups_of_today_date(&self) -> Result<i64, Error> {
        if !self.backup_directory.exists() {
            return Ok(1);
        }

        let timestamp = chrono::Local::now().format("%-m-%-d-%Y");
        let mut count = 0;
        for entry in fs::read_dir(&self.backup_directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if file_name.starts_with(&format!("{}-{}", self.name, timestamp)) {
                    count += 1;
                }
            }
        }
        Ok(count - 1)
    }

    fn does_tar_command_exist(&self) -> bool {
        let output = Command::new("tar").arg("--version").output();

        if output.is_err() {
            return false;
        }

        let the_output = output.unwrap();
        the_output.status.success()
    }

    fn does_zstd_command_exist(&self) -> bool {
        let output = Command::new("zstd").arg("--version").output();

        if output.is_err() {
            return false;
        }

        let the_output = output.unwrap();
        the_output.status.success()
    }

    fn does_zip_command_exist(&self) -> bool {
        let output = Command::new("zip").arg("--version").output();

        if output.is_err() {
            return false;
        }

        let the_output = output.unwrap();
        the_output.status.success()
    }
}
