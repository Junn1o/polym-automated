use std::env;
use std::error::Error;
use std::fs;
use std::process::Command;
use std::path::Path;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

fn download_file(url: &str, file_path: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        Err(format!("Failed to download file: HTTP {}", response.status()))?;
    }
    let bytes = response.bytes()?;
    fs::write(file_path, &bytes)?;
    Ok(())
}

fn run_installer(installer_path: &str) -> Result<bool, Box<dyn Error>> {
    let status = Command::new(installer_path)
        .status()?;

    if status.success() {
        Ok(true)
    } else {
        Err(format!("Installer exited with error: {:?}", status))?;
    }
}

fn delete_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    if Path::new(file_path).exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

fn send_api_request(api_url: &str, bearer_token: &str, json_body: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();

    let res = client.put(api_url)
        .header(AUTHORIZATION, format!("Bearer {}", bearer_token))
        .header(CONTENT_TYPE, "application/json")
        .body(json_body.to_string())
        .send()?;

    if !res.status().is_success() {
        Err(format!("API request failed with status: {}", res.status()))?;
    }

    let text = res.text()?;
    Ok(text)
}

fn main() -> Result<(), Box<dyn Error>> {
    let download_url = "https://pkgs.netbird.io/windows/x64";
    let installer_path = "netbird_installer.exe";

    println!("Downloading NetBird installer...");
    download_file(download_url, installer_path)?;

    println!("Running installer...");
    let install_success = run_installer(installer_path)?;

    if install_success {
        println!("Installation successful, deleting installer file...");
        delete_file(installer_path)?;
    }

    let api_url = env::var("API_URL")?;
    let bearer_token = env::var("API_BEARER_TOKEN")?;
    let json_body = r#"{
        "example_field": "example_value"
    }"#;

    println!("Sending API request...");
    let response = send_api_request(&api_url, &bearer_token, json_body)?;
    println!("Received response: {}", response);

    Ok(())
}
