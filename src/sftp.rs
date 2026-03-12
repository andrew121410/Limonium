use std::fs;
use std::io::{Error, ErrorKind, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use chrono::{NaiveDate, Utc};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use openssh::{Session, Stdio};
use openssh_sftp_client::Sftp;

use crate::backup::extract_date_from_file_name;

pub async fn login(user: String, host: String, port: Option<u16>, key_file: Option<&Path>) -> Result<Session, Error> {
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

pub async fn upload_file(session: &Session, path: &PathBuf, file_name: &str, remote_dir: &str, local_hash: &str) -> Result<(), Error> {
    let mut child = session
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

    let remote_create_dir_result = fs.create_dir(Path::new(remote_dir)).await;
    if remote_create_dir_result.is_err() {
        // Ignore error the directory probably already exists
    }

    let remote_file_result = sftp.create(Path::new(remote_dir).join(file_name)).await;

    if remote_file_result.is_err() {
        println!("{}", format!("Failed to create remote file: {}", remote_file_result.err().unwrap()).red());
        return Err(Error::new(ErrorKind::Other, "Failed to create remote file"));
    }

    let mut remote_file = remote_file_result.unwrap();
    let mut local_file = std::fs::File::open(path).unwrap();

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
    let mut command = session.command("sha256sum".to_string());
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

pub async fn delete_after_time(session: &Session, backup_name: &str, input: &str, remote_dir: &str) {
    // Deletes backups after a certain amount of time in the SFTP server
    // Example input: 1m = 1 month, 1w = 1 week, 1d = 1 day

    // Parse the input duration
    let (amount, unit) = input.split_at(input.len() - 1);
    let amount: i64 = amount.parse().unwrap_or(0);

    // Get the current date
    let current_date = Utc::now().naive_utc().date();

    // For each file in the backup directory
    let file_names = list_files(session, remote_dir).await.unwrap();

    for file_name in file_names {
        let date = extract_date_from_file_name(&file_name.to_string());

        // If the file name does not contain a date or does not start with the backup name, skip it
        if date.is_empty() || !file_name.starts_with(backup_name) {
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
                    delete_file(session, &file_name, remote_dir).await.unwrap_or_else(|e| {
                        eprintln!("Error deleting file: {}", e);
                    });
                }
            }
            "w" => {
                if days_difference > amount * 7 {
                    // Delete the file
                    delete_file(session, &file_name, remote_dir).await.unwrap_or_else(|e| {
                        eprintln!("Error deleting file: {}", e);
                    });
                }
            }
            "d" => {
                if days_difference > amount {
                    // Delete the file
                    delete_file(session, &file_name, remote_dir).await.unwrap_or_else(|e| {
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

async fn list_files(session: &Session, remote_dir: &str) -> Result<Vec<String>, Error> {
    // Just use ls command
    let mut command = session.command("ls".to_string());
    command.arg(remote_dir);

    // Execute the command and capture the output
    let output = command.output().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Check if the command was successful
    if !output.status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("ls command failed: {}", output.status),
        ));
    }

    // Convert the output bytes to a string
    let output_str = String::from_utf8(output.stdout).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // Split the output by newline characters and collect the file names
    let file_names = output_str.split('\n').map(|s| s.to_string()).collect();

    Ok(file_names)
}

async fn delete_file(session: &Session, file_name: &str, remote_dir: &str) -> Result<(), Error> {
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

