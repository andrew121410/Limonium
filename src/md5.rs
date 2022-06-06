use std::env::temp_dir;
use std::process::Command;

pub fn get_md5sum() -> String {
    let output = Command::new("md5sum")
        .arg("theServer.jar")
        .current_dir(temp_dir())
        .output()
        .expect("MD5Sum command output is error")
        .stdout;
    let string_output= String::from_utf8(output).expect("Invalid utf8");
    let string_vector: Vec<&str> = string_output.split(" ").collect();
    return string_vector.into_iter().nth(0).unwrap().to_string();
}