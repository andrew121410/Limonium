use std::env::temp_dir;
use std::process;
use std::process::Command;
use colored::Colorize;

pub fn validate_the_hash(hash: &Hash, tmp_jar_name: &String) -> bool {
    if hash.validate_hash(&tmp_jar_name).unwrap() {
        println!("{} {}", format!("{}", &hash.algorithm.to_uppercase()), format!("hash validation succeeded on jar!").green().bold());
        return true;
    } else {
        // If the hash didn't match then exit
        println!("{} {} {}", format!("{}", &hash.algorithm.to_uppercase()), format!("hash validation failed!").red().bold(), format!("{}", tmp_jar_name).yellow());

        // Print the difference between the hashes
        let expected_hash = &hash.hash;
        let hash_of_tmp_jar = hash.get_hash_from_tmp_jar(&tmp_jar_name).unwrap();
        println!("{} {} {}", format!("Expected").yellow(), format!("{}", expected_hash).green(), format!("but got").yellow());
        println!("{} {}", format!("{}", hash_of_tmp_jar).red(), format!("instead!").yellow());
        println!();
        println!();
        println!("{}", format!("Aborting...").red().bold());

        process::exit(102);
    }
}

// Gets the sha256 hash of the jar in the temp directory
pub fn get_sha256sum(jar_name: &String) -> String {
    let output = Command::new("sha256sum")
        .arg(&jar_name)
        .current_dir(temp_dir())
        .output()
        .expect("SHA256 command output has an error")
        .stdout;
    let string_output = String::from_utf8(output).expect("Invalid utf8");
    let string_vector: Vec<&str> = string_output.split(" ").collect();
    return string_vector.into_iter().nth(0).unwrap().to_string();
}

// Gets the md5 hash of the jar in the temp directory
pub fn get_md5sum(jar_name: &String) -> String {
    let output = Command::new("md5sum")
        .arg(&jar_name)
        .current_dir(temp_dir())
        .output()
        .expect("MD5Sum command output has an error")
        .stdout;
    let string_output = String::from_utf8(output).expect("Invalid utf8");
    let string_vector: Vec<&str> = string_output.split(" ").collect();
    return string_vector.into_iter().nth(0).unwrap().to_string();
}

pub struct Hash {
    pub algorithm: String,
    pub hash: String,
}

impl Hash {
    pub fn new(algorithm: String, hash: String) -> Self {
         Self {
            algorithm,
            hash,
        }
    }

    pub fn validate_hash(&self, jar_name: &String) -> Option<bool> {
        return match self.algorithm.as_str() {
            "sha256" => {
                Some(self.hash == get_sha256sum(&jar_name))
            }
            "md5" => {
                Some(self.hash == get_md5sum(&jar_name))
            }
            _ => {
                None
            }
        }
    }

    pub fn get_hash_from_tmp_jar(&self, jar_name: &String) -> Option<String> {
        return match self.algorithm.as_str() {
            "sha256" => {
                Some(get_sha256sum(&jar_name))
            }
            "md5" => {
                Some(get_md5sum(&jar_name))
            }
            _ => {
                None
            }
        }
    }
}