use std::env::current_dir;
use std::fs;
use std::io::{Error, ErrorKind, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use colored::Colorize;

pub enum BackupFormat {
    TarGz,
    Zip,
}

pub struct Backup {
    name: String,
    directory_to_backup: String,
    backup_directory: PathBuf,
    backup_format: BackupFormat,
    exclude: Option<String>,
}

impl Backup {
    pub fn new(name: String, directory_to_backup: String, backup_directory: PathBuf, backup_format: BackupFormat, exclude: Option<String>) -> Self {
        Backup {
            name,
            directory_to_backup,
            backup_directory,
            backup_format,
            exclude,
        }
    }

    pub fn backup(&self) -> Result<(), Error> {
        let timestamp = chrono::Local::now().format("%-m-%-d-%Y");

        let extension = match self.backup_format {
            BackupFormat::TarGz => "tar.gz",
            BackupFormat::Zip => "zip",
        };

        match self.backup_format {
            BackupFormat::TarGz => {
                if !self.does_tar_command_exist() {
                    return Err(Error::new(ErrorKind::Other, "The tar command does not exist. Please install it and try again."));
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
        let mut cmd: Command = Command::new("tar");
        match self.backup_format {
            BackupFormat::TarGz => {
                // You may exclude files and folders by splitting them with a : (colon)
                // Example: "logs:plugins/dynmap"
                if let Some(exclude) = &self.exclude {
                    let exclude = exclude.split(":");
                    println!("exclude: {:?}", exclude);

                    for exclude in exclude {
                        cmd.arg(format!("--exclude={}", exclude));
                    }
                }

                cmd.arg("-czf").arg(&backup_path);

                // You may backup multiple folders by splitting them with a : (colon)
                // Example: "world:world_nether:world_the_end"
                let folders_to_backup = self.directory_to_backup.split(":");
                println!("folders_to_backup: {:?}", folders_to_backup);

                for folder in folders_to_backup {
                    cmd.arg(&folder);
                }
            }
            BackupFormat::Zip => {
                cmd = Command::new("zip");
                cmd.arg("-r").arg(&backup_path);

                // You may backup multiple folders by splitting them with a : (colon)
                // Example: "world:world_nether:world_the_end"
                let folders_to_backup = self.directory_to_backup.split(":");
                println!("folders_to_backup: {:?}", folders_to_backup);

                for folder in folders_to_backup {
                    cmd.arg(&folder);
                }

                // You may exclude files and folders by splitting them with a : (colon)
                // Example: "logs:plugins/dynmap"
                if let Some(exclude) = &self.exclude {
                    let exclude = exclude.split(":");
                    println!("exclude: {:?}", exclude);

                    for exclude in exclude {
                        cmd.arg(format!("-x {}", exclude));
                    }
                }
            }
        };

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
        let mut combined_backup_path = self.backup_directory.join(format!("{}-{}-{}-bundle.{}", &self.name, timestamp, how_many_backups_of_today_date, extension));

        cmd = Command::new("tar");
        match self.backup_format {
            BackupFormat::TarGz => {
                cmd.arg("-czf").arg(&combined_backup_path);
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

    fn does_zip_command_exist(&self) -> bool {
        let output = Command::new("zip").arg("--version").output();

        if output.is_err() {
            return false;
        }

        let the_output = output.unwrap();
        the_output.status.success()
    }
}
