use std::fs;
use std::io::{Error, ErrorKind, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct Backup {
    name: String,
    server_directory: PathBuf,
    backup_directory: PathBuf,
}

impl Backup {
    pub fn new(name: String, server_directory: PathBuf, backup_directory: PathBuf) -> Self {
        Backup {
            name,
            server_directory,
            backup_directory,
        }
    }

    pub fn backup_tar_gz(&self) -> Result<(), Error> {
        let timestamp = chrono::Local::now().format("%-m-%-d-%Y");

        let backup_path = self.backup_directory.join(format!("{}-{}-temp.tar.gz", &self.name, timestamp));
        let hash_path = self.backup_directory.join(format!("{}-{}_hash.txt", &self.name, timestamp));

        if !self.backup_directory.exists() {
            fs::create_dir_all(&self.backup_directory)?;
            println!("The backup directory did not exist, so it was created at {}", &self.backup_directory.display());
        }

        // Create a gzip-compressed tar archive of the Minecraft server files
        let mut tar_cmd = Command::new("tar");
        tar_cmd.arg("-czf").arg(&backup_path);
        tar_cmd.arg("-C").arg(&self.server_directory);
        tar_cmd.arg(".");
        let tar_output = tar_cmd.output()?;
        if !tar_output.status.success() {
            return Err(Error::new(ErrorKind::Other, "Failed to create backup archive of Minecraft server files"));
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

        // Create a new gzip-compressed tar archive containing the backup archive and the hash file
        let mut combined_backup_path = backup_path.clone();
        let how_many_backups_of_today_date = self.get_how_many_backups_of_today_date()?;
        combined_backup_path.set_file_name(format!("{}-{}-{}.tar.gz", &self.name, timestamp, how_many_backups_of_today_date));

        let mut tar_cmd = Command::new("tar");
        tar_cmd.arg("-czf").arg(&combined_backup_path);
        tar_cmd.arg("-C").arg(&self.backup_directory);
        tar_cmd.arg(&backup_path.file_name().unwrap());
        tar_cmd.arg(&hash_path.file_name().unwrap());
        let tar_output = tar_cmd.output()?;
        if !tar_output.status.success() {
            return Err(Error::new(ErrorKind::Other, "Failed to create combined backup archive of Minecraft server files and hash file"));
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
}
