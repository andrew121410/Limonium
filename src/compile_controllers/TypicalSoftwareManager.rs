use crate::download_controllers;
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::io::BufRead;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

pub struct SoftwareConfig {
    pub repo_url: String,
    pub branch: Option<String>,
    pub build_command: String,
    pub jar_regex: String,
    pub jar_location: String,
}

pub async fn handle_software(config: SoftwareConfig, compile_path: &PathBuf, path: &mut String) {
    let software_path = compile_path.join(config.repo_url.split('/').last().unwrap().replace(".git", ""));
    if !software_path.exists() {
        git_clone(&config.repo_url, &config.branch, &compile_path).await;
    } else {
        git_pull(&software_path).await;
    }

    // Print software path
    println!("{}", format!("compile_path: {}", compile_path.display()).cyan());
    println!("{}", format!("Software path: {}", software_path.display()).cyan());

    build(&software_path, &config.build_command, path, &config.jar_regex, &config.jar_location);
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

    let main_command_path = software_path.join(main_command);
    if !main_command_path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Build command not found"));
    }

    // Main command path
    println!("{}", format!("Main command path: {}", main_command_path.display()).cyan());

    // Ensure the main command is executable
    ensure_executable(&main_command_path)?;

    // Convert the main command to Unix format
    convert_to_unix_format(&main_command_path)?;

    let mut command = Command::new(main_command_path);
    command.args(&args)
        .current_dir(software_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Debug print for the full command
    println!("Full command: {:?}", command);

    let mut process = command.spawn()?;
    process.wait()
}

fn build(software_path: &PathBuf, build_command: &str, path: &String, jar_regex: &str, jar_location: &str) {
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

    let libs_dir = software_path.join(jar_location);

    println!("{}", format!("Looking for the JAR file in {}", libs_dir.display()).cyan());

    let jar_files: Vec<PathBuf> = download_controllers::find_jar_files(&libs_dir, &Regex::new(jar_regex).unwrap());

    if jar_files.is_empty() {
        eprintln!("{}", "No JAR files found in the libs directory.".red());
        return;
    }

    jar_files.iter().for_each(|jar_file| {
        println!("{}", format!("Found JAR file: {}", jar_file.display()).cyan());
    });

    let our_jar_file = jar_files.iter().find(|&jar_file| {
        let file_name = jar_file.file_name().unwrap().to_string_lossy();
        println!("{}", format!("Checking file: {}", file_name).cyan());

        !file_name.contains("-sources") && !file_name.contains("-javadoc")
    });

    match our_jar_file {
        Some(jar_path) => {
            fs::rename(&jar_path, &path).expect("Failed to move the JAR file");
            println!(
                "{}",
                format!("Moved {} to {}", jar_path.file_name().unwrap().to_str().unwrap(), path.as_str()).green()
            );
        }
        None => {
            eprintln!("{}", "No matching JAR file found in the libs directory.".red());
        }
    }

    let duration = start.elapsed();
    println!(
        "{}",
        format!("Build completed in {:.2?} seconds", duration).cyan()
    );
}