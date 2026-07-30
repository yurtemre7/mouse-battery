use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    Hover,
    Icon,
}

impl Default for DisplayMode {
    fn default() -> Self {
        DisplayMode::Hover
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub time_delta: u64,
    pub display_mode: DisplayMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            time_delta: 300, // 5 minutes default
            display_mode: DisplayMode::Hover,
        }
    }
}

impl AppConfig {
    fn get_config_path() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("de", "yurtemre", "SteelMouse")?;
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).ok()?;
        Some(config_dir.join("config.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::get_config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::get_config_path() {
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, content);
            }
        }
    }
}
