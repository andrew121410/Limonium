use crate::download_controllers;
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

// PlotSquared
// plotsquared
pub(crate) struct PlotSquaredAPI;

impl PlotSquaredAPI {
    pub async fn handle_plotsquared(compile_path: &PathBuf, path: &mut String) {
        let plotsquared_path = compile_path.join("PlotSquared");
        if !plotsquared_path.exists() {
            fs::create_dir(&plotsquared_path).expect("Failed to create PlotSquared folder?");
            PlotSquaredAPI::git_clone(&compile_path).await; // We give the compile_path to the git_clone function instead of the plotsquared_path
        } else {
            // If the folder already exists, we will skip the git clone and just update the files using git pull
            PlotSquaredAPI::git_pull(&plotsquared_path).await;
        }

        PlotSquaredAPI::build(&plotsquared_path, path);
    }

    pub async fn git_clone(compile_path: &PathBuf) {
        let mut command = Command::new("git");
        command.arg("clone")
            .arg("https://github.com/IntellectualSites/PlotSquared.git")
            .current_dir(compile_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let mut process = command.spawn().expect("Failed to start git clone");
        let status = process.wait().expect("Failed to wait on git clone");

        if status.success() {
            println!("{}", format!("Cloned PlotSquared Repository").green());
        } else {
            eprintln!("{}", format!("Failed to clone PlotSquared Repository").red());
        }
    }

    pub async fn git_pull(compile_path: &PathBuf) {
        let mut command = Command::new("git");
        command.arg("pull")
            .current_dir(compile_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let mut process = command.spawn().expect("Failed to start git pull");
        let status = process.wait().expect("Failed to wait on git pull");

        if status.success() {
            println!("{}", format!("Pulled PlotSquared Repository to get the latest changes!").green());
        } else {
            eprintln!("{}", format!("Failed to pull PlotSquared Repository").red());
        }
    }

    pub fn build(compile_path: &PathBuf, path: &String) {
        println!(
            "{} {}",
            format!("Please wait patiently while PlotSquared compiles for you!").yellow(),
            format!(
                "If there's no changes, PlotSquared will skip compiling.\n\
             If you have left the folder there from last time, it might reuse the previous build."
            ).red()
        );

        let start = Instant::now();

        let mut command = Command::new("./gradlew");
        command.arg("build")
            .current_dir(compile_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let mut process = command.spawn().expect("Failed to run Gradle build");
        let status = process.wait().expect("Failed to wait on Gradle build");

        if status.success() {
            println!("{}", format!("PlotSquared compiled successfully!").green());

            let libs_dir = compile_path.join("Bukkit/build/libs/");

            println!("{}", format!("Looking for the JAR file in {}", libs_dir.display()).cyan());

            let jar_files: Vec<PathBuf> = download_controllers::find_jar_files(&libs_dir, &Regex::new(r"plotsquared-.*\.jar").unwrap());

            if jar_files.is_empty() {
                eprintln!("{}", "No JAR files found in the libs directory.".red());
                return;
            }

            // Debugging output
            jar_files.iter().for_each(|jar_file| {
                println!("{}", format!("Found JAR file: {}", jar_file.display()).cyan());
            });

            // Regex to match the JAR file excluding -sources and -javadoc
            let our_jar_file = jar_files.iter().find(|&jar_file| {
                let file_name = jar_file.file_name().unwrap().to_string_lossy();
                println!("{}", format!("Checking file: {}", file_name).cyan());

                // If it doesn't contain -sources or -javadoc, we will use it
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
        } else {
            eprintln!("{}", format!("PlotSquared failed to compile.").red());
        }

        let duration = start.elapsed();
        println!(
            "{}",
            format!("Build completed in {:.2?} seconds", duration).cyan()
        );
    }
}