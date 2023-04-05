use std::{env, process};
use std::collections::HashMap;
use std::env::temp_dir;
use std::time::Instant;

use colored::Colorize;

use crate::{api, hashutils};
use crate::hashutils::Hash;

pub async fn download_jar(project: &String, version: &String, path: &mut String) {
    let start = Instant::now();

    // Verify the project is actually a thing
    let map = get_all_types().await;
    if !map.contains_key(project.clone().as_str()) {
        println!("{} {} {}", format!("ServerJars.com -> Project").red(), format!("{}", &project).yellow(), format!("does not exist!").red());
        process::exit(102);
    }
    let server_jar = map.get(project.clone().as_str()).unwrap();

    let mut download_link = String::from("https://serverjars.com/api/fetchJar/");
    download_link.push_str(&server_jar.typea);
    download_link.push_str("/");
    download_link.push_str(&project);
    if !version.eq("latest") {
        download_link.push_str("/");
        download_link.push_str(&version);
    }

    let tmp_jar_name = api::download_jar_to_temp_dir(&download_link).await;

    let mut jar_details_url = String::from("https://serverjars.com/api/fetchDetails/");
    jar_details_url.push_str(&server_jar.typea);
    jar_details_url.push_str("/");
    jar_details_url.push_str(&project);
    if !version.eq("latest") {
        jar_details_url.push_str("/");
        jar_details_url.push_str(&version);
    }

    let jar_details_text = reqwest::get(&jar_details_url).await.unwrap().text().await.unwrap();
    let jar_details: FetchDetails = match serde_json::from_str(jar_details_text.as_str()) {
        Ok(v) => v,
        Err(_e) => {
            println!("{}", format!("ServersJars.com -> Failed to parse JSON (Most likely the version you requested doesn't exist)").red());
            println!("{} {} {}", format!("FYI: You can use").yellow(), format!("latest").green(), format!("as a version to get the latest version").yellow());
            println!("{} {} {} {}", format!("Example:").blue().bold(), format!("./limonium paper").purple(), format!("latest").green(), format!("--server-jars-com").purple());
            process::exit(102);
        }
    };

    // Verify the hash (all md5)
    let hash: Hash = Hash {
        algorithm: "md5".to_string(),
        hash: jar_details.response.md5.to_string(),
    };

    hashutils::validate_the_hash(&hash, temp_dir().as_path(), &tmp_jar_name, true);

    // If the path is empty then use the default
    if path.is_empty() {
        path.push_str(&jar_details.response.file);
    }

    // Move the jar to the correct location
    api::copy_jar_from_temp_dir_to_dest(&tmp_jar_name, &path);

    let duration = start.elapsed().as_millis().to_string();
    println!("{} {} {}", format!("ServerJars.com -> Successfully downloaded").green(), format!("{}", &project).blue().bold(), format!("from ServerJars.com").green());
    println!("{} {} {} {}", format!("Downloaded JAR:").green().bold(), format!("{}", &path.as_str()).blue().bold(), format!("Time In Milliseconds:").purple().bold(), format!("{}", &duration).yellow().bold());
}

pub async fn get_all_types() -> HashMap<String, AServerJar> {
    let text: String = reqwest::get("https://serverjars.com/api/fetchTypes/").await.unwrap().text().await.unwrap();
    let fetch_types: FetchTypes = serde_json::from_str(text.as_str()).unwrap();

    let mut map: HashMap<String, AServerJar> = HashMap::new();

    // Bedrock
    for x in fetch_types.response.bedrock {
        map.insert(x.clone(), AServerJar {
            name: x.clone(),
            typea: "bedrock".to_string(),
        });
    }

    // Modded
    for x in fetch_types.response.modded {
        map.insert(x.clone(), AServerJar {
            name: x.clone(),
            typea: "modded".to_string(),
        });
    }

    // Proxies
    for x in fetch_types.response.proxies {
        map.insert(x.clone(), AServerJar {
            name: x.clone(),
            typea: "proxies".to_string(),
        });
    }

    // Servers
    for x in fetch_types.response.servers {
        map.insert(x.clone(), AServerJar {
            name: x.clone(),
            typea: "servers".to_string(),
        });
    }

    // Vanilla
    for x in fetch_types.response.vanilla {
        map.insert(x.clone(), AServerJar {
            name: x.clone(),
            typea: "vanilla".to_string(),
        });
    }

    return map;
}

pub struct AServerJar {
    pub name: String,
    pub typea: String,
}

// https://serverjars.com/api/fetchTypes/
#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct FetchTypes {
    #[serde(rename = "status")]
    status: String,

    #[serde(rename = "response")]
    response: FetchTypesResponse,
}

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct FetchTypesResponse {
    #[serde(rename = "bedrock")]
    bedrock: Vec<String>,

    #[serde(rename = "modded")]
    modded: Vec<String>,

    #[serde(rename = "proxies")]
    proxies: Vec<String>,

    #[serde(rename = "servers")]
    servers: Vec<String>,

    #[serde(rename = "vanilla")]
    vanilla: Vec<String>,
}

// https://serverjars.com/api/fetchDetails/servers/paper/1.19.2
#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct FetchDetails {
    #[serde(rename = "status")]
    status: String,

    #[serde(rename = "response")]
    response: FetchDetailsResponse,
}

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct FetchDetailsResponse {
    #[serde(rename = "version")]
    version: String,

    #[serde(rename = "file")]
    file: String,

    #[serde(rename = "md5")]
    md5: String,

    #[serde(rename = "built")]
    built: i64,

    #[serde(rename = "stability")]
    stability: String,
}