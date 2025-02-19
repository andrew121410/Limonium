use clap::ArgMatches;
use colored::Colorize;
use std::ascii::AsciiExt;
use std::env;
use std::process::Command;
use crate::ensurer;

mod spigotmc;
mod plotsquared;
mod TypicalSoftwareManager;
mod mcmmo;
mod coreprotect;

pub(crate) struct CompileController;

impl CompileController {
    pub(crate) async fn handle_compile(compile_matches: &ArgMatches) {
        let current_dir_path_buffer = env::current_dir().unwrap();
        let current_path = current_dir_path_buffer.as_path();

        // Let us create a new directory for named "limonium-compile"
        let compile_dir = current_path.join("limonium-compile");
        if !compile_dir.exists() {
            std::fs::create_dir(&compile_dir).unwrap();
        }

        let software = compile_matches.get_one::<String>("software").unwrap();
        let temp = String::from("");
        let mut path_string = compile_matches.get_one::<String>("path").unwrap_or(&temp).to_string();

        let optional_version = compile_matches.get_one::<String>("version");
        let optional_branch = compile_matches.get_one::<String>("branch"); // We need to implement this
        let our_optional_branch = optional_branch.map(|branch| branch.to_string());

        // Check if Java is installed on the system
        if !ensurer::Ensurer::is_installed(&ensurer::Program::Java) {
            println!("{}", "Java is not installed on your system. Please install Java and try again.".red());
            return;
        }

        if software.eq_ignore_ascii_case("spigot") {
            // Version is required for spigot
            if optional_version.is_none() {
                println!("{}", "--version <version> is required for spigot".red());
                return;
            }

            spigotmc::SpigotAPI::handle_spigot(&compile_dir, optional_version.unwrap(), &mut path_string);
        } else if software.eq_ignore_ascii_case("PlotSquared") {
            plotsquared::PlotSquaredAPI::handle_plotsquared(&compile_dir, &mut path_string, our_optional_branch).await;
        } else if software.eq_ignore_ascii_case("mcMMO") {
            mcmmo::mcMMOAPI::handle_mcmmo(&compile_dir, &mut path_string, our_optional_branch).await;
        }else if software.eq_ignore_ascii_case("CoreProtect") {
            // Version is required for CoreProtect
            if optional_version.is_none() {
                println!("{}", "--version <version> is required for CoreProtect".red());
                return;
            }

            coreprotect::CoreProtectAPI::handle_coreprotect(&compile_dir, &mut path_string, our_optional_branch, optional_version.unwrap()).await;
    } else {
            println!("{}", format!("Unknown software: {}", software).red());
        }
    }
}