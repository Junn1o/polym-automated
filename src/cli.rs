use std::error::Error;
use std::process::Command;

// TODO: xác nhận lại cú pháp chính xác sau khi cài NetBird CLI trên máy thật.
pub fn netbird_up(management_url: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("netbird")
        .arg("up")
        .arg("--management-url")
        .arg(management_url)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        Err(format!("netbird up failed: {}{}", stdout, stderr))?;
    }

    Ok(stdout)
}

pub fn netbird_status() -> Result<String, Box<dyn Error>> {
    let output = Command::new("netbird").arg("status").output()?;

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
