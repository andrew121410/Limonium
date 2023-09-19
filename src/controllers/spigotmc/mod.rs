pub mod bungeecord;

use std::fs;
use std::process::Command;
use std::string::String;
use std::time::Instant;

use colored::Colorize;

// Unfortunately there’s no download site for Spigot. So we have to use BuildTools to compile it...
pub struct SpigotAPI;

impl SpigotAPI {
    pub fn download_build_tools() {
        fs::create_dir_all("./lmtmp/").expect("Failed to create lmtmp folder?");

        let _output = Command::new("wget")
            .arg("-O")
            .arg("./lmtmp/BuildTools.jar")
            .arg("https://hub.spigotmc.org/jenkins/job/BuildTools/lastSuccessfulBuild/artifact/target/BuildTools.jar")
            .output()
            .expect("Downloading BuildTools failed?");

        println!("{}", format!("Installed BuildTools.jar to ./lmtmp/").yellow())
    }

    pub fn run_build_tools(version: &String, path: &String) {
        println!("{} {}", format!("Please wait patiently while BuildTools compiles Spigot for you!").yellow(), format!("If there's no changes BuildTools will skip compiling \n\r If you have left the folder there from last time").red());

        let start = Instant::now();

        let _output = Command::new("java")
            .arg("-jar")
            .arg("BuildTools.jar")
            .arg("--rev")
            .arg(&version)
            .arg("--compile-if-changed")
            .current_dir("./lmtmp/")
            .output()
            .expect("Hmm");

        let mut copy_from = String::from("./lmtmp/spigot-");
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