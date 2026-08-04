use hidapi::HidApi;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use chrono::Utc;
use crate::log;

pub fn run_diagnostic(api: &mut HidApi) -> Value {
    log::log("Running HID mouse diagnostic scan...");
    let _ = api.refresh_devices();

    let mut device_records = Vec::new();
    let mut count = 0;

    for dev_info in api.device_list() {
        if dev_info.vendor_id() == 0x1038 {
            count += 1;
            let pid = dev_info.product_id();
            let iface = dev_info.interface_number();
            let prod = dev_info.product_string().unwrap_or("SteelSeries Device").to_string();
            let path_str = dev_info.path().to_string_lossy().to_string();

            log::log(&format!(
                "Diagnostic scan: found PID=0x{:04X} iface={} prod='{}'",
                pid, iface, prod
            ));

            let mut probed_commands = Vec::new();

            if let Ok(device) = dev_info.open_device(api) {
                for &cmd in &[0xD2u8, 0x92u8, 0xAAu8, 0xB2u8] {
                    // Drain buffer prior to sending command
                    let mut drain = [0u8; 64];
                    while device.read_timeout(&mut drain, 0).unwrap_or(0) > 0 {}

                    let mut req = [0u8; 64];
                    req[0] = 0x00;
                    req[1] = cmd;
                    if cmd == 0xAA {
                        req[2] = 0x01;
                    }

                    let write_ok = device.write(&[0x00, cmd]).is_ok()
                        || device.write(&req).is_ok()
                        || device.send_feature_report(&[0x00, cmd]).is_ok();

                    let mut responses = Vec::new();
                    if write_ok {
                        for attempt in 0..5 {
                            let mut res = [0u8; 64];
                            match device.read_timeout(&mut res, 200) {
                                Ok(n) if n >= 2 => {
                                    responses.push(json!({
                                        "attempt": attempt,
                                        "length": n,
                                        "raw_bytes": &res[..n]
                                    }));
                                }
                                _ => break,
                            }
                        }
                    }

                    probed_commands.push(json!({
                        "command": format!("0x{:02X}", cmd),
                        "write_success": write_ok,
                        "responses": responses
                    }));

                    std::thread::sleep(Duration::from_millis(50));
                }
            } else {
                log::log(&format!("  open_device failed for path={:?}", path_str));
            }

            device_records.push(json!({
                "vendor_id": "0x1038",
                "product_id": format!("0x{:04X}", pid),
                "product_name": prod,
                "interface": iface,
                "path": path_str,
                "probed_commands": probed_commands
            }));
        }
    }

    log::log(&format!("Diagnostic scan complete. Scanned {} devices.", count));

    json!({
        "steelmouse_version": env!("CARGO_PKG_VERSION"),
        "os": env::consts::OS,
        "arch": env::consts::ARCH,
        "timestamp": Utc::now().to_rfc3339(),
        "devices_found": count,
        "devices": device_records
    })
}

pub fn save_diagnostic_file(custom_path: Option<&Path>) -> Result<PathBuf, String> {
    let mut api = HidApi::new().map_err(|e| format!("Failed to init HidApi: {}", e))?;
    let report = run_diagnostic(&mut api);

    let target_path = match custom_path {
        Some(p) => p.to_path_buf(),
        None => {
            let filename = format!("steelmouse_diagnostic_{}.json", Utc::now().format("%Y%m%d_%H%M%S"));
            if let Some(desktop) = directories::UserDirs::new().and_then(|u| u.desktop_dir().map(|d| d.to_path_buf())) {
                desktop.join(filename)
            } else {
                PathBuf::from(filename)
            }
        }
    };

    let formatted_json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    if let Some(parent) = target_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::write(&target_path, formatted_json).map_err(|e| format!("Failed to write file {}: {}", target_path.display(), e))?;
    log::log(&format!("Diagnostic report saved to {}", target_path.display()));
    Ok(target_path)
}

pub fn open_file_in_file_manager(path: &Path) {
    let path_str = path.to_string_lossy();
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .args(&["/select,", &path_str])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .args(&["-R", &path_str])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            let _ = std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_report_json_schema() {
        if let Ok(mut api) = HidApi::new() {
            let report = run_diagnostic(&mut api);
            assert!(report.get("steelmouse_version").is_some());
            assert!(report.get("os").is_some());
            assert!(report.get("timestamp").is_some());
            assert!(report.get("devices").is_some());
        }
    }

    #[test]
    fn test_save_diagnostic_file_custom_path() {
        let temp_dir = std::env::temp_dir();
        let target_file = temp_dir.join("steelmouse_test_diag.json");
        if target_file.exists() {
            let _ = std::fs::remove_file(&target_file);
        }

        if let Ok(saved_path) = save_diagnostic_file(Some(&target_file)) {
            assert!(saved_path.exists());
            let content = std::fs::read_to_string(&saved_path).unwrap();
            assert!(content.contains("steelmouse_version"));
            let _ = std::fs::remove_file(saved_path);
        }
    }
}
