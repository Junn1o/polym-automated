use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn download_file(url: &str, file_path: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        Err(format!("Failed to download file: HTTP {}", response.status()))?;
    }
    let bytes = response.bytes()?;
    fs::write(file_path, &bytes)?;
    Ok(())
}

pub fn run_installer(installer_path: &str) -> Result<bool, Box<dyn Error>> {
    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Start-Process -FilePath '{}' -Verb RunAs -Wait",
                installer_path
            ),
        ])
        .status()?;

    if status.success() {
        Ok(true)
    } else {
        Err(format!("Installer exited with error: {:?}", status))?
    }
}

pub fn delete_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    if Path::new(file_path).exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}
