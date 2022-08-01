use std::env::temp_dir;
use std::process::Command;

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
}