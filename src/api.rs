use std::error::Error;

use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct InviteRequest {
    pub email: String,
    pub name: String,
    pub role: String,
    pub auto_groups: Vec<String>,
    pub expires_in: u64,
}

#[derive(Deserialize)]
pub struct InviteResponse {
    pub id: String,
    pub email: String,
    pub invite_token: String,
}

pub fn create_invite(
    api_url: &str,
    bearer_token: &str,
    request: &InviteRequest,
) -> Result<InviteResponse, Box<dyn Error>> {
    let client = Client::new();

    let res = client
        .post(api_url)
        .header(AUTHORIZATION, format!("Token {}", bearer_token))
        .header(CONTENT_TYPE, "application/json")
        .json(request)
        .send()?;

    if !res.status().is_success() {
        Err(format!("Invite API request failed with status: {}", res.status()))?;
    }

    let invite: InviteResponse = res.json()?;
    Ok(invite)
}

pub fn build_invite_url(domain: &str, invite_token: &str) -> String {
    format!("https://{}/invite/{}", domain, invite_token)
}
