use std::env::temp_dir;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};

use colored::Colorize;

pub struct LogSearch {
    days_back: u64,
    to_search: String,
}

impl LogSearch {
    pub fn new(days_back: u64, to_search: String) -> LogSearch {
        LogSearch {
            days_back,
            to_search,
        }
    }

    pub fn context(&self, lines_before: u64, lines_after: u64) {
        // Create a temporary directory inside the "logs" folder to hold the log files
        let temp_dir = match fs::create_dir("logs/temp") {
            Ok(_) => PathBuf::from("logs/temp"),
            Err(e) => {
                PathBuf::from("logs/temp")
            }
        };

        // Get the current date and time
        let now = SystemTime::now();

        // Calculate the date and time 4 days ago
        let four_days_ago = now - Duration::from_secs(self.days_back * 24 * 60 * 60);

        // Loop over all the days within the search range
        let mut current_time = now;
        while current_time > four_days_ago {
            let date_string = self.date_string(current_time);
            let mut file_index = 1;

            // Loop over all the log files for this day
            loop {
                let logs_dir = PathBuf::from("logs");
                let log_filename = format!("{}-{}.log", date_string, file_index);
                let log_path = logs_dir.join(&log_filename);

                let gz_filename = format!("{}-{}.log.gz", date_string, file_index);
                let gz_path = logs_dir.join(&gz_filename);

                if !gz_path.exists() {
                    println!("This file does not exist: {:?}", gz_path);
                    break;
                }

                fs::copy(&gz_path, temp_dir.join(&gz_filename)).unwrap_or_else(|e| {
                    eprintln!("Error copying file: {}", e);
                    std::process::exit(1);
                });

                // uncompress the file using the gzip -d command
                let output = Command::new("gzip")
                    .arg("-d")
                    .arg(temp_dir.join(&gz_filename))
                    .output()
                    .expect("failed to execute process");

                let file_to_read = File::open(temp_dir.join(&log_filename)).unwrap();

                // Read the file line by line
                let reader = BufReader::new(file_to_read);
                let mut matching_line_number = 0;
                let mut context = vec![];
                for (line_number, line) in reader.lines().enumerate() {
                    if let Ok(line) = line {
                        // Check if the line contains the keyword
                        if line.contains(self.to_search.as_str()) {
                            // Save the line number of the matching line
                            matching_line_number = line_number;

                            // Add the matching line to the context
                            context.push((line_number, line.clone()));
                        } else if !context.is_empty()
                            && line_number >= matching_line_number.checked_sub(lines_before as usize).unwrap_or(0)
                            && line_number <= matching_line_number + lines_after as usize
                        {
                            // Add the line to the context if it's within the range of lines to print
                            context.push((line_number, line.clone()));

                            // Remove the oldest line from the context if it's too long
                            if context.len() > (lines_before + lines_after + 1) as usize {
                                context.remove(0);
                            }
                        }
                    }
                }

                // Print the context for the matching line, if any
                if !context.is_empty() {
                    println!("{}", format!("Found matching line in {}", log_filename).bright_magenta());
                    for (line_number, line) in context {
                        println!("{}: {}", line_number + 1, line);
                    }
                }

                // Move to the next log file for this day
                file_index += 1;
            }

            // Move to the previous day
            current_time -= Duration::from_secs(24 * 60 * 60);
            file_index = 1;
        }

        // Delete the temporary directory
        fs::remove_dir_all(temp_dir).unwrap_or_else(|e| {
            eprintln!("Error deleting directory: {}", e);
        });
    }

    // Helper function to format a SystemTime value as a date string
    fn date_string(&self, time: SystemTime) -> String {
        use chrono::{Datelike, DateTime, TimeZone, Utc};

        let date_time = DateTime::<Utc>::from(time);
        date_time.format("%Y-%m-%d").to_string()
    }
}
