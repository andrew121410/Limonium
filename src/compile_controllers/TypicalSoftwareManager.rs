use crate::{download_controllers, ensurer, file_utils};
use colored::Colorize;
use regex::Regex;
use std::io::BufRead;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{fs, thread};

pub struct SoftwareConfig {
    pub repo_url: String,
    pub branch: Option<String>,
    pub build_command: String,
    pub jar_regex: String,
    pub jar_location: String,
    pub before_building_function: Option<Box<dyn Fn(&PathBuf)>>,
    pub custom_find_jar_function: Option<Box<dyn Fn(&PathBuf) -> Result<PathBuf, std::io::Error>>>,
    pub after_building_function: Option<Box<dyn Fn(&PathBuf)>>,
    pub delete_after_building: bool,
}

pub async fn handle_software(config: SoftwareConfig, compile_path: &PathBuf, path: &mut String) {
    // Check if Git is installed on the system
    if !ensurer::Ensurer::is_installed(&ensurer::Program::Git) {
        eprintln!("{}", "Git is not installed on your system. Please install Git and try again.".red());
        return;
    }

    let software_path = compile_path.join(config.repo_url.split('/').last().unwrap().replace(".git", ""));
    if !software_path.exists() {
        git_clone(&config.repo_url, &config.branch, &compile_path).await;
    } else {
        git_pull(&software_path).await;
    }

    // Print software path
    println!("{}", format!("compile_path: {}", compile_path.display()).cyan());
    println!("{}", format!("software_path: {}", software_path.display()).cyan());

    // Call before_building_function if provided
    if let Some(before_build) = &config.before_building_function {
        before_build(&software_path);
    }

    build(&software_path, &config.build_command, path, &config.jar_regex, &config.jar_location, &config.custom_find_jar_function);

    // Call after_building_function if provided
    if let Some(after_build) = &config.after_building_function {
        after_build(&software_path);
    }

    // Delete the software path if delete_after_building is true
    if config.delete_after_building {
        fs::remove_dir_all(&software_path).expect("Failed to delete software path");
        println!("{}", format!("Deleted software path: {}", software_path.display()).green());
    }
}

async fn git_clone(repo_url: &str, branch: &Option<String>, compile_path: &PathBuf) {
    let mut command = Command::new("git");
    command.arg("clone")
        .arg(repo_url)
        .current_dir(compile_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Some(branch_name) = branch {
        command.arg("--branch").arg(branch_name);
    }

    let mut process = command.spawn().expect("Failed to start git clone");
    let status = process.wait().expect("Failed to wait on git clone");

    if status.success() {
        println!("{}", format!("Cloned repository: {}", repo_url).green());
    } else {
        eprintln!("{}", format!("Failed to clone repository: {}", repo_url).red());
    }
}

async fn git_pull(software_path: &PathBuf) {
    let mut command = Command::new("git");
    command.arg("pull")
        .current_dir(software_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut process = command.spawn().expect("Failed to start git pull");
    let status = process.wait().expect("Failed to wait on git pull");

    if status.success() {
        println!("{}", format!("Pulled latest changes for repository").green());
    } else {
        eprintln!("{}", format!("Failed to pull latest changes for repository").red());
    }
}

fn convert_to_unix_format(file_path: &PathBuf) -> Result<(), std::io::Error> {
    let status = Command::new("dos2unix")
        .arg(file_path)
        .status()?;

    if status.success() {
        println!("{}", format!("Converted file to Unix format: {}", file_path.display()).green());
        Ok(())
    } else {
        println!("{}", format!("Failed to convert file to Unix format: {}", file_path.display()).red());
        println!("{}", format!("Please install dos2unix and try again.").red());
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to convert file to Unix format"))
    }
}

fn ensure_executable(file_path: &PathBuf) -> Result<(), std::io::Error> {
    let metadata = fs::metadata(file_path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755); // Set execute permissions
    fs::set_permissions(file_path, permissions)
}

fn run_build_command(software_path: &PathBuf, build_command: &str) -> Result<std::process::ExitStatus, std::io::Error> {
    let mut parts = build_command.split_whitespace();
    let main_command = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();

    // Check if the main command is maven
    if main_command.contains("mvn") && !ensurer::Ensurer::is_installed(&ensurer::Program::Mvn) {
        eprintln!("{}", format!("Maven is not installed on your system. Please install Maven and try again.").red());
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Maven is not installed"));
    }

    // If the executable is in the software path like gradlew NOT maven though
    let main_command_path = software_path.join(main_command);
    if main_command_path.exists() {
        // Main command path
        println!("{}", format!("Main command path: {}", main_command_path.display()).cyan());

        // Ensure the main command is executable
        ensure_executable(&main_command_path)?;

        // Convert the main command to Unix format
        convert_to_unix_format(&main_command_path)?;
    }

    let mut command = Command::new(if main_command_path.exists() {
        main_command_path.as_path()
    } else {
        Path::new(main_command)
    });
    command.args(&args)
        .current_dir(software_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Debug print for the full command
    println!("Full command: {:?}", command);

    let mut process = command.spawn()?;
    process.wait()
}

fn build(software_path: &PathBuf, build_command: &str, path: &String, jar_regex: &str, jar_location: &str, custom_find_jar_function: &Option<Box<dyn Fn(&PathBuf) -> Result<PathBuf, std::io::Error>>>) {
    println!(
        "{} {}",
        format!("Please wait patiently while the software compiles for you!").yellow(),
        format!(
            "If there's no changes, the software will skip compiling.\n\
             If you have left the folder there from last time, it might reuse the previous build."
        ).red()
    );

    let start = Instant::now();

    let status = match run_build_command(software_path, build_command) {
        Ok(status) => status,
        Err(e) => {
            eprintln!("{}", format!("Failed to run build command: {:?}", e).red());
            return;
        }
    };

    if !status.success() {
        eprintln!("{}", format!("Software failed to compile.").red());
        return;
    }

    println!("{}", format!("Software compiled successfully!").green());

    let mut jar_file: PathBuf;

    // If there's a custom find_jar_function, use it
    if let Some(custom_find_jar) = custom_find_jar_function {
        jar_file = custom_find_jar(software_path).expect("Failed to find JAR file");
    } else {
        let libs_dir = software_path.join(jar_location);
        println!("{}", format!("Looking for the JAR files in {}", libs_dir.display()).cyan());
        let jar_files: Vec<PathBuf> = file_utils::find_jar_files(&libs_dir, &Regex::new(jar_regex).unwrap());

        if jar_files.is_empty() {
            eprintln!("{}", "No JAR files found in the libs directory.".red());
            return;
        }

        jar_files.iter().for_each(|jar_file| {
            println!("{}", format!("Found JAR file: {}", jar_file.file_name().expect("Couldn't get file_name").to_str().unwrap().to_string()).cyan());
        });

        // If there is only one JAR file, use it
        if jar_files.len() == 1 {
            jar_file = jar_files[0].clone();
        }

        // If there are multiple JAR files, let's try to sort them
        let did_we_find_one_auto: Option<PathBuf> = jar_files.clone().into_iter().find(|jar_file| {
            let file_name = jar_file.file_name().unwrap().to_string_lossy();
            println!("{}", format!("Checking file: {}", file_name).cyan());

            !file_name.contains("-sources") && !file_name.contains("-javadoc")
        });

        if did_we_find_one_auto.is_some() {
            jar_file = did_we_find_one_auto.unwrap();
        } else {
            // Else let the user choose let there be like a timer to pick within 30 seconds else fail
            println!("{}", "Multiple JAR files found. Please choose one:".yellow());
            for (index, jar_file) in jar_files.iter().enumerate() {
                println!("{}: {}", index, jar_file.file_name().expect("Couldn't get file_name").to_str().unwrap());
            }

            let (tx, rx) = mpsc::channel();
            let thread = thread::spawn(move || {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("Failed to read line");
                tx.send(input).expect("Failed to send input");
            });

            let input = match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(input) => input,
                Err(_) => {
                    eprintln!("{}", "No input received within 30 seconds. Exiting.".red());
                    return;
                }
            };

            let index: usize = match input.trim().parse() {
                Ok(index) => index,
                Err(_) => {
                    eprintln!("{}", "Failed to parse input".red());
                    return;
                }
            };

            if index >= jar_files.len() {
                eprintln!("{}", "Invalid index".red());
                return;
            }

            jar_file = jar_files[index].clone();
            println!("{}", format!("Found Correct JAR file: {}", jar_file.file_name().unwrap().to_str().unwrap().to_string()).green());
        }
    };

    println!("{}", format!("Found Correct JAR file: {}", jar_file.file_name().unwrap().to_str().unwrap().to_string()).green());

    // Copy the JAR file to the output path
    let path = PathBuf::from(path);
    fs::copy(&jar_file, &path).expect("Failed to copy the JAR file");
    println!("{}", format!("Copied JAR file to: {}", path.display()).green());

    let duration = start.elapsed();
    println!(
        "{}",
        format!("Build completed in {:.2?} seconds", duration).cyan()
    );
}