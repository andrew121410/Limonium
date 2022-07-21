use reqwest::header;

pub struct GithubUtils;

impl GithubUtils {
    pub async fn get_latest_tag(user: &str, repo: &str) -> Option<String> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            "rust-reqwest/limonium".parse().unwrap(),
        );
        let response = reqwest::Client::new()
            .get(&format!("https://api.github.com/repos/{}/{}/tags", user, repo))
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
}

#[derive(Deserialize, Default)]
struct Tag {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    name: Option<String>,
}