use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::string::String;
use std::time::Instant;

use colored::Colorize;
use crate::ensurer;

pub struct SpigotAPI;

impl SpigotAPI {
    pub fn handle_spigot(compile_path: &PathBuf, version: &String, path: &mut String) {
        ensurer::Ensurer::ensure_programs(&[ensurer::Program::Wget]);
        ensurer::Ensurer::ensure_programs(&[ensurer::Program::Java]);

        // In the limonium-compile folder, we will create a new folder called spigot
        let spigot_path = compile_path.join("spigot");
        if !spigot_path.exists() {
            fs::create_dir(&spigot_path).expect("Failed to create spigot folder?");
        }

        SpigotAPI::download_build_tools(&spigot_path);
        SpigotAPI::run_build_tools(&spigot_path, &version, &path);
    }

    pub fn download_build_tools(compile_path: &PathBuf) {
        let _output = Command::new("wget")
            .arg("-O")
            .arg("./BuildTools.jar")
            .arg("https://hub.spigotmc.org/jenkins/job/BuildTools/lastSuccessfulBuild/artifact/target/BuildTools.jar")
            .current_dir(compile_path)
            .output()
            .expect("Downloading BuildTools failed?");

        println!("{}", format!("Downloaded BuildTools.jar").green());
    }

    pub fn run_build_tools(compile_path: &PathBuf, version: &String, path: &String) {
        println!("{} {}", format!("Please wait patiently while BuildTools compiles Spigot for you!").yellow(), format!("If there's no changes BuildTools will skip compiling \n\r If you have left the folder there from last time").red());

        let start = Instant::now();

        let mut command = Command::new("java")
            .arg("-jar")
            .arg("BuildTools.jar")
            .arg("--rev")
            .arg(&version)
            .arg("--compile-if-changed")
            .current_dir(compile_path)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to run BuildTools?");

        let output = command.stdout.take().unwrap();
        let reader = BufReader::new(output);

        for line in reader.lines() {
            println!("{}", line.unwrap());
        }

        command.wait().expect("Failed to wait on child process");

        let compile_path_string = compile_path.to_str().unwrap();
        let mut copy_from = String::from(compile_path_string.to_string() + "/spigot-");
        copy_from.push_str(&version);
        copy_from.push_str(".jar");

        fs::copy(&copy_from, &path).expect("Failed copying jar?");

        let duration = start.elapsed().as_secs().to_string();

        let mut string = String::from("Done installing ");
        string.push_str(&path);
        string.push_str(" took ");
        string.push_str(&duration);
        string.push_str(" seconds");

        println!("{}", format!("{}", &string).green());
    }
}