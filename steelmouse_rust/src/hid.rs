use hidapi::{DeviceInfo, HidApi, HidDevice};
use std::fmt;
use crate::devices::{self, BatteryKind};

const STEELSERIES_VENDOR_ID: u16 = 0x1038;
const CHARGING_FLAG: u8 = 0x80;
const READ_TIMEOUT_MS: i32 = 300;

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub name: String,
    pub level: Option<u8>,
    pub is_charging: bool,
}

impl fmt::Display for BatteryInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level_str = self
            .level
            .map(|l| format!("{}%", l))
            .unwrap_or_else(|| "N/A".to_string());
        let status_str = if self.is_charging {
            "Charging"
        } else {
            "Discharging"
        };
        write!(f, "{}: {} ({})", self.name, level_str, status_str)
    }
}

struct ActiveDevice {
    device: HidDevice,
    name: String,
    kind: BatteryKind,
}

pub struct MouseManager {
    mock_mode: bool,
    mock_level: u8,
    mock_charging: bool,
    api: Option<HidApi>,
    cached_device: Option<ActiveDevice>,
}

fn get_interface_number(info: &DeviceInfo) -> i32 {
    if info.interface_number() >= 0 {
        return info.interface_number();
    }
    
    // Parse interface/collection number from Windows device path (e.g. "\\?\hid#vid_1038&pid_1838&mi_03#...")
    let path_str = info.path().to_string_lossy().to_lowercase();
    if let Some(pos) = path_str.find("mi_") {
        let rest = &path_str[pos + 3..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
        if let Ok(num) = i32::from_str_radix(&num_str, 16) {
            return num;
        }
    }
    if let Some(pos) = path_str.find("col") {
        let rest = &path_str[pos + 3..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
        if let Ok(num) = i32::from_str_radix(&num_str, 16) {
            return num;
        }
    }
    -1
}

impl MouseManager {
    pub fn new(mock_mode: bool) -> Self {
        let api = if !mock_mode {
            HidApi::new().ok()
        } else {
            None
        };

        Self {
            mock_mode,
            mock_level: 85,
            mock_charging: false,
            api,
            cached_device: None,
        }
    }

    pub fn fetch_battery(&mut self) -> Result<BatteryInfo, String> {
        if self.mock_mode {
            let current = self.mock_level;
            if self.mock_charging {
                self.mock_level = if current >= 100 { 100 } else { current + 1 };
            } else {
                self.mock_level = if current <= 5 { 100 } else { current - 1 };
            }

            return Ok(BatteryInfo {
                name: "SteelSeries Aerox 3 Wireless (Mock)".to_string(),
                level: Some(self.mock_level),
                is_charging: self.mock_charging,
            });
        }

        // 1. Try polling existing open cached device handle
        if let Some(active) = &self.cached_device {
            if let Some((level, is_charging)) = Self::query_device_battery(&active.device, active.kind) {
                return Ok(BatteryInfo {
                    name: active.name.clone(),
                    level,
                    is_charging,
                });
            }
        }

        // 2. Cached device failed or not connected yet - clear cache & scan devices
        self.cached_device = None;

        let api = match &mut self.api {
            Some(api) => {
                let _ = api.refresh_devices();
                api
            }
            None => {
                let new_api = HidApi::new().map_err(|e| format!("Failed to init hidapi: {}", e))?;
                self.api = Some(new_api);
                self.api.as_mut().unwrap()
            }
        };

        for device_info in api.device_list() {
            if device_info.vendor_id() == STEELSERIES_VENDOR_ID {
                let pid = device_info.product_id();
                let profile = devices::get_profile(STEELSERIES_VENDOR_ID, pid);

                if let Some(prof) = profile {
                    if let Some(kind) = prof.battery_kind {
                        // Filter interfaces by endpoint matching
                        let iface = get_interface_number(device_info);
                        if prof.endpoint != 0 && iface >= 0 && iface != prof.endpoint as i32 {
                            continue;
                        }

                        if let Ok(device) = device_info.open_device(api) {
                            if let Some((level, is_charging)) = Self::query_device_battery(&device, kind) {
                                let name = prof.name.to_string();
                                self.cached_device = Some(ActiveDevice {
                                    device,
                                    name: name.clone(),
                                    kind,
                                });

                                return Ok(BatteryInfo {
                                    name,
                                    level,
                                    is_charging,
                                });
                            }
                        }
                    }
                } else {
                    // Fallback for unlisted SteelSeries devices
                    if let Ok(device) = device_info.open_device(api) {
                        if let Some((level, is_charging)) = Self::query_generic_fallback(&device) {
                            let name = device_info
                                .product_string()
                                .unwrap_or("SteelSeries Mouse")
                                .to_string();

                            return Ok(BatteryInfo {
                                name,
                                level,
                                is_charging,
                            });
                        }
                    }
                }
            }
        }

        Err("No supported SteelSeries mouse found".to_string())
    }

    fn query_device_battery(device: &HidDevice, kind: BatteryKind) -> Option<(Option<u8>, bool)> {
        match kind {
            BatteryKind::AeroxPrime { command } => {
                let pkt = [0x00, command];

                if device.write(&pkt).is_err() {
                    if device.send_feature_report(&pkt).is_err() {
                        let feat_pkt = [0x02, command];
                        if device.send_feature_report(&feat_pkt).is_err() {
                            return None;
                        }
                    }
                }

                let mut res = [0u8; 64];
                if let Ok(read_len) = device.read_timeout(&mut res, READ_TIMEOUT_MS) {
                    if read_len >= 1 {
                        return Self::decode_aerox_prime_battery(&res[..read_len]);
                    }
                }
            }
            BatteryKind::Rival3Or650 => {
                let pkt = [0x00, 0xAA, 0x01];

                if device.write(&pkt).is_err() {
                    if device.send_feature_report(&pkt).is_err() {
                        let feat_pkt = [0x02, 0xAA, 0x01];
                        if device.send_feature_report(&feat_pkt).is_err() {
                            return None;
                        }
                    }
                }

                let mut res = [0u8; 64];
                if let Ok(read_len) = device.read_timeout(&mut res, READ_TIMEOUT_MS) {
                    if read_len >= 1 {
                        return Self::decode_rival3_650_battery(&res[..read_len]);
                    }
                }
            }
        }
        None
    }

    fn decode_aerox_prime_battery(res: &[u8]) -> Option<(Option<u8>, bool)> {
        // Robust byte detection handling both Windows (leading Report ID 0x00) and macOS/Linux
        let byte_to_check = if res.len() >= 2 && res[0] == 0x00 && res[1] > 0 {
            res[1]
        } else if !res.is_empty() && res[0] > 0 {
            res[0]
        } else if res.len() >= 2 && res[1] > 0 {
            res[1]
        } else {
            return None;
        };

        let is_charging = (byte_to_check & CHARGING_FLAG) != 0;
        let raw_val = byte_to_check & !CHARGING_FLAG;
        if raw_val > 0 {
            let level = ((raw_val.saturating_sub(1)) as u16 * 5).min(100) as u8;
            Some((Some(level), is_charging))
        } else {
            Some((None, is_charging))
        }
    }

    fn decode_rival3_650_battery(res: &[u8]) -> Option<(Option<u8>, bool)> {
        if res.is_empty() {
            return None;
        }
        let (level_byte, charging_byte) = if res[0] == 0x00 && res.len() >= 3 {
            (res[1], res[2])
        } else if res.len() >= 3 {
            (res[0], res[2])
        } else {
            (res[0], 0)
        };

        let level = Some(level_byte.min(100));
        let is_charging = charging_byte != 0;
        Some((level, is_charging))
    }

    fn query_generic_fallback(device: &HidDevice) -> Option<(Option<u8>, bool)> {
        if let Some(res) = Self::query_device_battery(device, BatteryKind::AeroxPrime { command: 0xD2 }) {
            return Some(res);
        }
        if let Some(res) = Self::query_device_battery(device, BatteryKind::AeroxPrime { command: 0x92 }) {
            return Some(res);
        }
        Self::query_device_battery(device, BatteryKind::Rival3Or650)
    }
}
