#![windows_subsystem = "windows"]
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

pub fn download_file(
    url: &str,
    file_path: &str,
    mut on_progress: impl FnMut(u64, Option<u64>),
) -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(300))
        .build()?;

    let mut response = client
        .get(url)
        .send()
        .map_err(|e| format!("Không kết nối được tới server để tải file: {}", e))?;

    if !response.status().is_success() {
        Err(format!("Failed to download file: HTTP {}", response.status()))?;
    }

    let total = response.content_length();
    let mut file = File::create(file_path)?;
    let mut buf = [0u8; 8192];
    let mut downloaded: u64 = 0;

    loop {
        let n = response
            .read(&mut buf)
            .map_err(|e| format!("Mất kết nối trong khi tải file: {}", e))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;
        on_progress(downloaded, total);
    }

    Ok(())
}

#[cfg(windows)]
pub fn run_installer(installer_path: &str) -> Result<bool, Box<dyn Error>> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        GetExitCodeProcess, WaitForSingleObject, INFINITE,
    };
    use windows::Win32::UI::Shell::{
        ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
    };

    let verb = HSTRING::from("runas");
    let file = HSTRING::from(installer_path);
    let params = HSTRING::from("/S");

    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        nShow: 1, // SW_SHOWNORMAL
        ..Default::default()
    };

    unsafe {
        ShellExecuteExW(&mut info)
            .map_err(|e| format!("Không chạy được trình cài đặt (cần quyền admin): {}", e))?;

        if info.hProcess.is_invalid() {
            return Err("Không lấy được tiến trình trình cài đặt".into());
        }

        WaitForSingleObject(info.hProcess, INFINITE);

        let mut exit_code: u32 = 0;
        GetExitCodeProcess(info.hProcess, &mut exit_code)
            .map_err(|e| format!("Không đọc được kết quả trình cài đặt: {}", e))?;
        let _ = CloseHandle(info.hProcess);

        if exit_code == 0 {
            Ok(true)
        } else {
            Err(format!("Installer exited with code: {}", exit_code))?
        }
    }
}

pub fn delete_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    if Path::new(file_path).exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}
