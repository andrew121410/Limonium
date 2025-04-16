use std::process::{exit, Command};
use colored::Colorize;

#[derive(Debug)]
pub enum Program {
    Java,
    Zip,
    Unzip,
    Wget,
    Dos2Unix,
    Mvn,
    Git,
    Tar,
    Sha256Sum,
    Ls,
    Zstd,
    Gzip
}

impl Program {
    /// Returns the command name and the arguments used for version checking.
    fn command(&self) -> (&'static str, &'static [&'static str]) {
        match self {
            Program::Java => ("java", &["-version"]),
            Program::Zip => ("zip", &["-v"]),
            Program::Unzip => ("unzip", &["-v"]),
            Program::Wget => ("wget", &["--version"]),
            Program::Dos2Unix => ("dos2unix", &["--version"]),
            Program::Mvn => ("mvn", &["--version"]),
            Program::Git => ("git", &["--version"]),
            Program::Tar => ("tar", &["--version"]),
            Program::Sha256Sum => ("sha256sum", &["--version"]),
            Program::Ls => ("ls", &["--version"]),
            Program::Zstd => ("zstd", &["--version"]),
            Program::Gzip => ("gzip", &["--version"]),
        }
    }
}

pub struct Ensurer;

impl Ensurer {
    /// Checks whether the specified program is installed by running its command.
    pub fn is_installed(program: &Program) -> bool {
        let (cmd, args) = program.command();
        let output = Command::new(cmd).args(args).output();
        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    /// Checks a list of programs and prints their installation status.
    pub fn check_programs(programs: &[Program]) {
        for program in programs {
            let status = if Self::is_installed(program) {
                "installed"
            } else {
                "not installed"
            };
            println!("{:?} is {}", program, status);
        }
    }

    /// Ensures that all specified programs are installed.
    /// If any program is missing, prints an error and exits the application.
    pub fn ensure_programs(programs: &[Program]) {
        for program in programs {
            if !Self::is_installed(program) {
                let (cmd, _) = program.command();
                eprintln!("{}", format!("Error: {} is required but not installed. Aborting.", cmd).red());
                exit(1);
            }
        }
    }
}
