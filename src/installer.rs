use std::error::Error;
use std::fs;
use std::path::Path;

pub fn download_file(url: &str, file_path: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        Err(format!("Failed to download file: HTTP {}", response.status()))?;
    }
    let bytes = response.bytes()?;
    fs::write(file_path, &bytes)?;
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

    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
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
