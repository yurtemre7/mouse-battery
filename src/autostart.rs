use std::env;
use std::path::PathBuf;
use crate::log;

pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("reg")
            .args(&["query", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "/v", "SteelMouse"])
            .output();
        if let Ok(out) = output {
            return out.status.success();
        }
        false
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs_home() {
            let plist_path = home.join("Library/LaunchAgents/com.steelmouse.plist");
            return plist_path.exists();
        }
        false
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        false
    }
}

pub fn set_autostart(enable: bool) -> Result<(), String> {
    let current_exe = env::current_exe().map_err(|e| e.to_string())?;
    let exe_str = current_exe.to_string_lossy().to_string();

    log::log(&format!("set_autostart(enable={}, exe='{}')", enable, exe_str));

    #[cfg(target_os = "windows")]
    {
        if enable {
            let status = std::process::Command::new("reg")
                .args(&[
                    "add",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    "SteelMouse",
                    "/t",
                    "REG_SZ",
                    "/d",
                    &format!("\"{}\"", exe_str),
                    "/f",
                ])
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                return Err("Failed to write Windows registry autostart key".to_string());
            }
        } else {
            let _ = std::process::Command::new("reg")
                .args(&[
                    "delete",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    "SteelMouse",
                    "/f",
                ])
                .status();
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs_home() {
            let launch_agents_dir = home.join("Library/LaunchAgents");
            let plist_path = launch_agents_dir.join("com.steelmouse.plist");
            if enable {
                let _ = std::fs::create_dir_all(&launch_agents_dir);
                let plist_content = format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.steelmouse.app</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
                    exe_str
                );
                std::fs::write(&plist_path, plist_content).map_err(|e| e.to_string())?;
            } else {
                if plist_path.exists() {
                    let _ = std::fs::remove_file(plist_path);
                }
            }
        }
    }

    Ok(())
}

fn dirs_home() -> Option<PathBuf> {
    directories::UserDirs::new().map(|u| u.home_dir().to_path_buf())
}
