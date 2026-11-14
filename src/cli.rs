use std::error::Error;
use std::path::Path;
use std::process::Command;

const FALLBACK_NETBIRD_PATH: &str = "C:\\Program Files\\NetBird\\netbird.exe";

fn netbird_binary() -> String {
    if Path::new(FALLBACK_NETBIRD_PATH).exists() {
        FALLBACK_NETBIRD_PATH.to_string()
    } else {
        "netbird".to_string()
    }
}

pub fn netbird_up(management_url: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new(netbird_binary())
        .arg("up")
        .arg("--management-url")
        .arg(management_url)
        .output()
        .map_err(|e| format!("Không chạy được netbird (đã cài NetBird CLI chưa?): {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        Err(format!("netbird up failed: {}{}", stdout, stderr))?;
    }

    Ok(stdout)
}

pub fn netbird_status() -> Result<String, Box<dyn Error>> {
    let output = Command::new(netbird_binary())
        .arg("status")
        .output()
        .map_err(|e| format!("Không chạy được netbird (đã cài NetBird CLI chưa?): {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        Err(format!("netbird status failed: {}{}", stdout, stderr))?;
    }

    Ok(stdout)
}

pub fn is_connected(status_output: &str) -> bool {
    status_output.to_lowercase().contains("connected")
}
