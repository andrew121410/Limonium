use reqwest::header;

pub struct Repo {
    pub user: String,
    pub repo: String,
}

impl Repo {
    pub fn new(name: &str, owner: &str) -> Self {
        Self {
            user: name.to_string(),
            repo: owner.to_string(),
        }
    }

    pub async fn get_latest_tag(&self) -> Option<String> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            "rust-reqwest/limonium".parse().unwrap(),
        );
        let response = reqwest::Client::new()
            .get(&format!("https://api.github.com/repos/{}/{}/tags", &self.user, &self.repo))
            .headers(headers)
            .send().await.unwrap();

        let text = response.text().await.unwrap();
        let tags: Vec<Tag> = serde_json::from_str(&text).expect("Failed to parse tags for github");

        let first_wrapped = tags.first();
        if first_wrapped.is_some() {
            let first = first_wrapped.unwrap();
            return Some(first.name.clone().expect("Getting tag name failed"));
        }
        return None;
    }

    pub fn get_download_link(&self, tag: &str, name_of_thing_to_download: &str) -> String {
        let status = self_update::backends::github::Update::configure()
            .repo_owner(&self.user)
            .repo_name(&self.repo)
            .target(&name_of_thing_to_download)
            .bin_name("na") // Not used, but required by the API
            .current_version("na") // Not used, but required by the API
            .build()
            .expect(format!("Building failed for {}/{}", &self.user, &self.repo).as_str());

        let latest_release = status.get_release_version(&tag).expect(format!("Getting release version failed for {}/{}", &self.user, &self.repo).as_str());
        let release_asset = latest_release.asset_for(&status.target()).expect(format!("Getting release asset failed for {}/{}", &self.user, &self.repo).as_str());

        return release_asset.download_url;
    }
}

#[derive(Deserialize, Default)]
struct Tag {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    name: Option<String>,
}