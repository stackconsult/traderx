use anyhow::Result;
use std::time::Duration;

/// Check that QuestDB is reachable and the `ticks` table exists.
pub async fn check_questdb(host: &str, http_port: u16) -> Result<bool> {
    let url = format!(
        "http://{}:{}/exec?query=SELECT+count()+FROM+ticks",
        host, http_port
    );
    let resp = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?
        .get(&url)
        .send()
        .await?;
    Ok(resp.status().is_success())
}
