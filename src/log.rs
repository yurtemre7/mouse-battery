// src/log.rs - File-based logger for Windows (since windows_subsystem="windows" hides stdout)
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn init() {
    // Write log next to the exe / in the user data dir
    let path = dirs_path();
    if let Ok(f) = OpenOptions::new().create(true).append(true).open(&path) {
        if let Ok(mut guard) = LOG_FILE.lock() {
            *guard = Some(f);
        }
    }
    log(&format!("\n\n===== SteelMouse started {} =====", chrono_now()));
    log(&format!("Log file: {}", path));
}

pub fn log(msg: &str) {
    // Always try stdout (works on macOS / cargo run)
    println!("[LOG] {}", msg);

    // Also write to file (essential on Windows where stdout is hidden)
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(f) = guard.as_mut() {
            let _ = writeln!(f, "[{}] {}", chrono_now(), msg);
            let _ = f.flush();
        }
    }
}

fn dirs_path() -> String {
    // Try to place the log alongside the exe
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("steelmouse.log").to_string_lossy().to_string();
        }
    }
    "steelmouse.log".to_string()
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Simple HH:MM:SS from unix timestamp
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    format!("{:02}:{:02}:{:02}", h, m, s)
}
