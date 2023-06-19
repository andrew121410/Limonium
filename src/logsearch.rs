use std::collections::HashSet;
use std::env::temp_dir;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, exit, Stdio};
use std::time::{Duration, SystemTime};

use colored::Colorize;

pub struct LogSearch {
    days_back: u64,
    to_search: String,
    logs_dir: PathBuf,
}

impl LogSearch {
    pub fn new(days_back: u64, to_search: String, logs_dir: PathBuf) -> LogSearch {
        LogSearch {
            days_back,
            to_search,
            logs_dir,
        }
    }

    pub fn context(&self, lines_before: u64, lines_after: u64) {
        // Create a temporary directory inside the "logs" folder to hold the log files
        let temp_dir_in_logs_folder = self.logs_dir.join("temp");
        fs::create_dir(&temp_dir_in_logs_folder).unwrap_or_else(|e| {
            eprintln!("Error creating directory: {}", e);
            std::process::exit(1);
        });

        // Get the current date and time
        let now = SystemTime::now();

        let days_back = now - Duration::from_secs(self.days_back * 24 * 60 * 60);

        // Loop over all the days within the search range
        let mut current_time = now;
        while current_time > days_back {
            let date_string = self.date_string(current_time);
            let mut file_index = 1;

            // Loop over all the log files for this day
            loop {
                let log_filename = format!("{}-{}.log", date_string, file_index);
                let log_path = self.logs_dir.join(&log_filename);

                let gz_filename = format!("{}-{}.log.gz", date_string, file_index);
                let gz_path = self.logs_dir.join(&gz_filename);

                if !gz_path.exists() {
                    break;
                }

                fs::copy(&gz_path, temp_dir_in_logs_folder.join(&gz_filename)).unwrap_or_else(|e| {
                    eprintln!("Error copying file: {}", e);
                    std::process::exit(1);
                });

                // uncompress the file using the gzip -d command
                let output = Command::new("gzip")
                    .arg("-d")
                    .arg(temp_dir_in_logs_folder.join(&gz_filename))
                    .output()
                    .expect("failed to execute process");

                let file_to_read = File::open(temp_dir_in_logs_folder.join(&log_filename)).unwrap();

                // Read the file line by line
                let reader = BufReader::new(file_to_read);
                let mut matching_line_number = 0;
                let mut context = vec![];
                let mut added_lines_before = HashSet::new();
                let mut added_lines_after = HashSet::new();
                // Loop over all the lines in the file
                for (line_number, line) in reader.lines().enumerate() {
                    if let Ok(line) = line {
                        // Check if the line contains the keyword
                        if line.contains(self.to_search.as_str()) {
                            // Save the line number of the matching line
                            matching_line_number = line_number;

                            // Add the lines before the matching line to the context
                            if lines_before >= 1 {
                                let mut reader2 = BufReader::new(File::open(
                                    temp_dir_in_logs_folder.join(&log_filename),
                                ).unwrap());

                                let mut temp_context = vec![];

                                // Start from the matching line number and go backwards
                                for line_number2 in (matching_line_number - lines_before as usize..matching_line_number).rev() {
                                    if let Some(Ok(line)) = reader2.by_ref().lines().nth(line_number2) {
                                        // Check if the line has already been added to the context
                                        if !added_lines_before.contains(&line) {
                                            temp_context.push((line_number2, line.clone()));

                                            // Add the line to the set of added lines
                                            added_lines_before.insert(line.clone());
                                        }
                                    } else {
                                        break;
                                    }
                                }

                                // Reverse the context so that it's in the correct order
                                temp_context.reverse();

                                // Add the context to the main context
                                context.extend(temp_context);
                            }

                            // Add the matching line to the context
                            context.push((line_number, line.clone()));

                            // Add the lines after the matching line to the context
                        } else if !context.is_empty() && line_number <= matching_line_number + lines_after as usize {
                            // Check if the line has already been added to the context
                            if !added_lines_after.contains(&line) {
                                context.push((line_number, line.clone()));

                                // Add the line to the set of added lines
                                added_lines_after.insert(line.clone());
                            }
                        }
                    }
                }

                // Write the context to a file if there is any
                if !context.is_empty() {
                    self.write_context_to_file(&context, log_filename.clone());
                }

                // Move to the next log file for this day
                file_index += 1;
            }

            // Move to the previous day
            current_time -= Duration::from_secs(24 * 60 * 60);
            file_index = 1;
        }

        // Delete the temporary directory
        fs::remove_dir_all(temp_dir_in_logs_folder).unwrap_or_else(|e| {
            eprintln!("Error deleting directory: {}", e);
        });

        // Open the file in nano
        self.open_in_nano();

        println!("Deleting limonium-log-search.txt...");
        // Delete the text file in the temporary directory
        fs::remove_file(temp_dir().join("limonium-log-search.txt")).unwrap_or_else(|e| {
            eprintln!("Error deleting file: {}", e);
        });
    }

    fn write_context_to_file(&self, context: &[(usize, String)], log_filename: String) {
        let file_path = temp_dir().join("limonium-log-search.txt");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .unwrap();

        writeln!(file, "NEW LOG FILE: {} (lines from this log file showed BELOW until next log file)", log_filename).unwrap();
        for (line_number, line) in context {
            writeln!(file, "{}: {}", line_number + 1, line).expect("Error writing to file");
        }
    }

    fn open_in_nano(&self) {
        let file_path = temp_dir().join("limonium-log-search.txt");
        let mut output = Command::new("nano")
            .arg(file_path)
            .stdin(Stdio::inherit())
            .spawn()
            .expect("failed to execute process");

        let status = output.wait().expect("failed to wait for process");

        if !status.success() {
            eprintln!("Error opening file in nano");
            exit(1);
        }
    }

    // Helper function to format a SystemTime value as a date string
    fn date_string(&self, time: SystemTime) -> String {
        use chrono::{Datelike, DateTime, TimeZone, Utc};

        let date_time = DateTime::<Utc>::from(time);
        date_time.format("%Y-%m-%d").to_string()
    }
}
