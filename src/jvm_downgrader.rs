use crate::{download_controllers, ensurer, file_utils};
use colored::Colorize;
use std::fs;
use std::path::{PathBuf};
use std::process::Command;

pub async fn run_jvm_downgrader(major_version: &String, input_jar: &PathBuf, output_jar : &PathBuf) {
    ensurer::Ensurer::ensure_programs(&[ensurer::Program::Java]);

    let jvm_downgrader_temp_dir = file_utils::get_or_create_limonium_dir().join("jvm_downgrader");
    fs::create_dir_all(&jvm_downgrader_temp_dir)
        .expect("Failed to create JVM Downgrader temp directory");

    // Copy the input jar to the temp directory
    let input_jar_path = jvm_downgrader_temp_dir.join("input.jar");
    fs::copy(&input_jar, &input_jar_path).expect("Failed to copy input jar");
    fs::remove_file(&input_jar).expect("Failed to delete downloaded jar from temp directory");

    let jvm_downgrader_download_link = "https://github.com/unimined/JvmDowngrader/releases/download/1.3.3/jvmdowngrader-1.3.3-all.jar".to_string();
    let jvm_downgrader_downloaded_jar = download_controllers::download_file_to_temp_dir_with_progress_bar(&jvm_downgrader_download_link, &".jar".to_string(), &file_utils::get_or_create_limonium_dir()).await;

    let current_path = jvm_downgrader_downloaded_jar.temp_file_path;
    let final_path = jvm_downgrader_temp_dir.join("jvmdowngrader.jar");
    fs::copy(&current_path, &final_path)
        .expect("Failed to copy JVM Downgrader to temp directory");
    fs::remove_file(&current_path)
        .expect("Failed to delete JVM Downgrader jar from temp directory");

    let output = Command::new("java")
        .arg("-jar")
        .arg("jvmdowngrader.jar")
        .arg("-c")
        .arg(&major_version)
        .arg("downgrade")
        .arg("-t")
        .arg("input.jar")
        .arg("output.jar")
        .current_dir(&jvm_downgrader_temp_dir)
        .output()
        .expect("Failed to execute JVM Downgrader");

    println!("JVM Downgrader output:");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("JVM Downgrader errors (if any):");
    println!("{}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        eprintln!(
            "JVM Downgrader failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = Command::new("java")
        .arg("-jar")
        .arg("jvmdowngrader.jar")
        .arg("-c")
        .arg(&major_version)
        .arg("shade")
        .arg("-p")
        .arg("shade/prefix/")
        .arg("-t")
        .arg("output.jar")
        .arg("new_output.jar")
        .current_dir(&jvm_downgrader_temp_dir)
        .output()
        .expect("Failed to execute JVM Downgrader");

    println!("JVM Downgrader output:");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("JVM Downgrader errors (if any):");
    println!("{}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        eprintln!(
            "JVM Downgrader failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let final_path = jvm_downgrader_temp_dir.join("new_output.jar");
    fs::copy(&final_path, &output_jar)
        .expect("Failed to copy JVM Downgrader output to temp directory");

    fs::remove_dir_all(&jvm_downgrader_temp_dir)
        .expect("Failed to delete JVM Downgrader temp directory");

    println!("{}", "JVM Downgrader was successful!".green().bold());
}
