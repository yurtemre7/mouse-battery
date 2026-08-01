use hidapi::{DeviceInfo, HidApi, HidDevice};
use crate::log;
use std::fmt;
use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use crate::devices::{self, BatteryKind};

const STEELSERIES_VENDOR_ID: u16 = 0x1038;
const CHARGING_FLAG: u8 = 0x80;
const READ_TIMEOUT_MS: i32 = 200;

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub name: String,
    pub level: Option<u8>,
    pub is_charging: bool,
    pub estimated_time: Option<String>,
}

impl fmt::Display for BatteryInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level_str = self
            .level
            .map(|l| format!("{}%", l))
            .unwrap_or_else(|| "N/A".to_string());
        let mut status_str = if self.is_charging {
            "Charging".to_string()
        } else {
            "Discharging".to_string()
        };

        if let Some(est) = &self.estimated_time {
            status_str.push_str(&format!(", {}", est));
        }

        write!(f, "{}: {} ({})", self.name, level_str, status_str)
    }
}

#[derive(Debug, Clone)]
pub struct BatterySample {
    pub timestamp: DateTime<Utc>,
    pub level: u8,
    pub is_charging: bool,
}

#[derive(Debug, Default)]
pub struct BatteryTracker {
    samples: VecDeque<BatterySample>,
    last_charging_state: Option<bool>,
}

impl BatteryTracker {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            last_charging_state: None,
        }
    }

    pub fn add_sample(&mut self, level: u8, is_charging: bool) {
        let now = Utc::now();

        if self.last_charging_state != Some(is_charging) {
            self.samples.clear();
            self.last_charging_state = Some(is_charging);
        }

        if self.samples.is_empty() || self.samples.back().map_or(true, |s| s.level != level) {
            self.samples.push_back(BatterySample {
                timestamp: now,
                level,
                is_charging,
            });

            if self.samples.len() > 20 {
                self.samples.pop_front();
            }
        }
    }

    #[cfg(test)]
    pub fn add_sample_with_time(&mut self, level: u8, is_charging: bool, timestamp: DateTime<Utc>) {
        if self.last_charging_state != Some(is_charging) {
            self.samples.clear();
            self.last_charging_state = Some(is_charging);
        }

        if self.samples.is_empty() || self.samples.back().map_or(true, |s| s.level != level) {
            self.samples.push_back(BatterySample {
                timestamp,
                level,
                is_charging,
            });

            if self.samples.len() > 20 {
                self.samples.pop_front();
            }
        }
    }

    pub fn estimate_time(&self) -> Option<String> {
        if self.samples.len() < 2 {
            return None;
        }

        let first = self.samples.front()?;
        let last = self.samples.back()?;

        let time_diff_sec = (last.timestamp - first.timestamp).num_seconds();
        if time_diff_sec < 5 {
            return None;
        }

        let is_charging = last.is_charging;
        let level_diff = (last.level as i32) - (first.level as i32);

        if is_charging {
            if level_diff <= 0 || last.level >= 100 {
                return None;
            }
            let rate_per_sec = (level_diff as f64) / (time_diff_sec as f64);
            let needed_percent = (100 - last.level) as f64;
            let total_sec_remaining = needed_percent / rate_per_sec;

            let minutes = (total_sec_remaining / 60.0).round() as i64;
            if minutes <= 0 {
                return None;
            }
            if minutes < 60 {
                Some(format!("Full in ~{}m", minutes))
            } else {
                let hours = minutes / 60;
                let mins = minutes % 60;
                Some(format!("Full in ~{}h {}m", hours, mins))
            }
        } else {
            if level_diff >= 0 || last.level == 0 {
                return None;
            }
            let rate_per_sec = (-level_diff as f64) / (time_diff_sec as f64);
            let total_sec_remaining = (last.level as f64) / rate_per_sec;

            let minutes = (total_sec_remaining / 60.0).round() as i64;
            if minutes <= 0 {
                return None;
            }
            if minutes < 60 {
                Some(format!("~{}m left", minutes))
            } else {
                let hours = minutes / 60;
                let mins = minutes % 60;
                Some(format!("~{}h {}m left", hours, mins))
            }
        }
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
    tracker: BatteryTracker,
}

fn get_interface_number(info: &DeviceInfo) -> i32 {
    if info.interface_number() >= 0 {
        return info.interface_number();
    }
    
    // Parse interface number from Windows device path string (e.g. mi_03 or &mi_03 or col03)
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
        log::log(&format!("MouseManager::new(mock_mode={})", mock_mode));
        let api = if !mock_mode {
            match HidApi::new() {
                Ok(a) => { log::log("HidApi::new() OK"); Some(a) }
                Err(e) => { log::log(&format!("HidApi::new() FAILED: {}", e)); None }
            }
        } else {
            None
        };

        Self {
            mock_mode,
            mock_level: 85,
            mock_charging: false,
            api,
            cached_device: None,
            tracker: BatteryTracker::new(),
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

            self.tracker.add_sample(self.mock_level, self.mock_charging);
            let estimated_time = self.tracker.estimate_time();

            return Ok(BatteryInfo {
                name: "SteelSeries Aerox 3 Wireless (Mock)".to_string(),
                level: Some(self.mock_level),
                is_charging: self.mock_charging,
                estimated_time,
            });
        }

        // 1. Try polling existing open cached device handle
        if let Some(active) = &self.cached_device {
            if let Some((level, is_charging)) = Self::query_device_battery(&active.device, active.kind) {
                if let Some(lvl) = level {
                    self.tracker.add_sample(lvl, is_charging);
                }
                let estimated_time = self.tracker.estimate_time();

                return Ok(BatteryInfo {
                    name: active.name.clone(),
                    level,
                    is_charging,
                    estimated_time,
                });
            }
        }

        // 2. Cached device failed or not connected yet - clear cache & scan devices
        self.cached_device = None;
        log::log("Scanning for SteelSeries HID devices...");

        let api = match &mut self.api {
            Some(api) => {
                let _ = api.refresh_devices();
                api
            }
            None => {
                let new_api = HidApi::new().map_err(|e| {
                    let msg = format!("Failed to init hidapi: {}", e);
                    log::log(&msg);
                    msg
                })?;
                self.api = Some(new_api);
                self.api.as_mut().unwrap()
            }
        };

        let mut steelseries_count = 0;
        for device_info in api.device_list() {
            if device_info.vendor_id() == STEELSERIES_VENDOR_ID {
                steelseries_count += 1;
                let pid = device_info.product_id();
                let iface = get_interface_number(device_info);
                log::log(&format!(
                    "  Found SteelSeries PID=0x{:04x} iface={} path={:?} product={:?}",
                    pid, iface, device_info.path(), device_info.product_string()
                ));

                let profile = devices::get_profile(STEELSERIES_VENDOR_ID, pid);

                if let Some(prof) = profile {
                    if let Some(kind) = prof.battery_kind {
                        // Skip non-matching interfaces to prevent OS driver locks and timeouts
                        if prof.endpoint != 0 && iface >= 0 && iface != prof.endpoint as i32 {
                            log::log(&format!("    Skipping: endpoint mismatch (need {}, got {})", prof.endpoint, iface));
                            continue;
                        }

                        match device_info.open_device(api) {
                            Err(e) => {
                                log::log(&format!("    open_device FAILED: {}", e));
                            }
                            Ok(device) => {
                                log::log(&format!("    open_device OK, querying kind={:?}", kind));
                                match Self::query_device_battery(&device, kind) {
                                    None => {
                                        log::log("    query_device_battery returned None");
                                    }
                                    Some((level, is_charging)) => {
                                        log::log(&format!("    SUCCESS level={:?} charging={}", level, is_charging));
                                        if let Some(lvl) = level {
                                            self.tracker.add_sample(lvl, is_charging);
                                        }
                                        let estimated_time = self.tracker.estimate_time();
                                        let name = prof.name.to_string();
                                        self.cached_device = Some(ActiveDevice { device, name: name.clone(), kind });
                                        return Ok(BatteryInfo { name, level, is_charging, estimated_time });
                                    }
                                }
                            }
                        }
                    } else {
                        log::log("    Profile found but no battery_kind (wired or unsupported)");
                    }
                } else {
                    // Fallback for unlisted SteelSeries devices
                    log::log("    No profile - trying generic fallback");
                    if let Ok(device) = device_info.open_device(api) {
                        if let Some((level, is_charging)) = Self::query_generic_fallback(&device) {
                            log::log(&format!("    Fallback SUCCESS level={:?} charging={}", level, is_charging));
                            if let Some(lvl) = level {
                                self.tracker.add_sample(lvl, is_charging);
                            }
                            let estimated_time = self.tracker.estimate_time();
                            let name = device_info
                                .product_string()
                                .unwrap_or("SteelSeries Mouse")
                                .to_string();
                            return Ok(BatteryInfo { name, level, is_charging, estimated_time });
                        }
                    }
                }
            }
        }

        let err = if steelseries_count == 0 {
            "No SteelSeries HID devices detected at all".to_string()
        } else {
            format!("Found {} SteelSeries devices but none returned battery data", steelseries_count)
        };
        log::log(&format!("fetch_battery error: {}", err));
        Err(err)
    }

    fn query_device_battery(device: &HidDevice, kind: BatteryKind) -> Option<(Option<u8>, bool)> {
        // Drain any stale bytes left in the HID read buffer (non-blocking, 0ms timeout)
        let mut drain_buf = [0u8; 64];
        while device.read_timeout(&mut drain_buf, 0).unwrap_or(0) > 0 {}

        match kind {
            BatteryKind::AeroxPrime { command } => {
                let mut req_64 = [0u8; 64];
                req_64[0] = 0x00;
                req_64[1] = command;

                let write_ok = device.write(&req_64).is_ok()
                    || device.write(&[0x00, command]).is_ok()
                    || device.send_feature_report(&[0x00, command]).is_ok();

                if !write_ok {
                    log::log(&format!("AeroxPrime write failed for cmd=0x{:02X}", command));
                    return None;
                }

                // Read up to 8 HID reports, skipping non-battery events.
                // A valid battery response starts with:
                //   - res[0] == command echo (e.g. 0xD2 / 210)
                //   - OR res[0] == 0x00 and res[1] == command echo (leading Report ID 0x00)
                let mut res = [0u8; 64];
                for attempt in 0..8 {
                    match device.read_timeout(&mut res, READ_TIMEOUT_MS) {
                        Ok(n) if n >= 2 => {
                            log::log(&format!("AeroxPrime attempt {} raw[0..8]={:?}", attempt, &res[..n.min(8)]));
                            let is_battery_response = res[0] == command
                                || (res[0] == 0x00 && n >= 3 && res[1] == command);
                            if is_battery_response {
                                if let Some(decoded) = Self::decode_aerox_prime_response(&res[..n]) {
                                    return Some(decoded);
                                }
                            }
                            log::log(&format!("  -> Skipping (res[0]=0x{:02X}, res[1]=0x{:02X})", res[0], res[1]));
                        }
                        Ok(0) | Err(_) => break, // Timeout or error - no more data
                        Ok(n) => { log::log(&format!("AeroxPrime short read: {} bytes", n)); break; }
                    }
                }
                None
            }
            BatteryKind::Rival3Or650 => {
                let mut req_64 = [0u8; 64];
                req_64[0] = 0x00;
                req_64[1] = 0xAA;
                req_64[2] = 0x01;

                let write_ok = device.write(&req_64).is_ok()
                    || device.write(&[0x00, 0xAA, 0x01]).is_ok()
                    || device.send_feature_report(&[0x00, 0xAA, 0x01]).is_ok();

                if !write_ok {
                    return None;
                }

                let mut res = [0u8; 64];
                if let Ok(read_len) = device.read_timeout(&mut res, READ_TIMEOUT_MS) {
                    if read_len >= 2 {
                        return Self::decode_rival3_650_response(&res[..read_len]);
                    }
                }
                None
            }
        }
    }
    fn decode_aerox_prime_response(res: &[u8]) -> Option<(Option<u8>, bool)> {
        if res.len() < 2 {
            return None;
        }

        // Response formats:
        //   Windows: [cmd_echo, 0x00, battery, ...] → battery at res[2]  (caller pre-validates res[1]==0x00)
        //   macOS:   [cmd_echo, battery]             → battery at res[1]  (2-byte short form)
        //   Windows with Report ID: [0x00, cmd_echo, 0x00, battery, ...] → battery at res[3]
        let battery_byte = if res.len() >= 4 && res[0] == 0x00 && res[2] == 0x00 {
            res[3] // [0x00, cmd, 0x00, battery, ...]
        } else if res.len() >= 3 && res[1] == 0x00 {
            res[2] // [cmd, 0x00, battery, ...] ← standard Windows form
        } else if res.len() >= 3 && res[0] == 0x00 {
            res[2] // [0x00, cmd, battery, ...]
        } else {
            res[1] // [cmd, battery] ← macOS short form
        };

        if battery_byte == 0 {
            return None;
        }

        let is_charging = (battery_byte & CHARGING_FLAG) != 0;
        let raw_val = battery_byte & !CHARGING_FLAG;
        let level = if raw_val > 0 {
            Some(((raw_val.saturating_sub(1)) as u16 * 5).min(100) as u8)
        } else {
            None
        };

        log::log(&format!("decode_aerox_prime: battery_byte=0x{:02X} raw_val={} level={:?} charging={}", battery_byte, raw_val, level, is_charging));
        Some((level, is_charging))
    }

    fn decode_rival3_650_response(res: &[u8]) -> Option<(Option<u8>, bool)> {
        if res.is_empty() {
            return None;
        }

        let (level_byte, charging_byte) = if res[0] == 0x00 && res.len() >= 4 {
            (res[1], res[3])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_aerox_prime_windows_format() {
        let res_100 = [0x00, 0x15, 21];
        let decoded = MouseManager::decode_aerox_prime_response(&res_100).expect("Should decode");
        assert_eq!(decoded.0, Some(100));
        assert_eq!(decoded.1, false);

        let res_85 = [0x00, 0x15, 18];
        let decoded = MouseManager::decode_aerox_prime_response(&res_85).expect("Should decode");
        assert_eq!(decoded.0, Some(85));
        assert_eq!(decoded.1, false);

        let res_50_charging = [0x00, 0x15, 11 | 0x80];
        let decoded = MouseManager::decode_aerox_prime_response(&res_50_charging).expect("Should decode");
        assert_eq!(decoded.0, Some(50));
        assert_eq!(decoded.1, true);

        let res_off = [0x00, 0x15, 0];
        let decoded = MouseManager::decode_aerox_prime_response(&res_off);
        assert!(decoded.is_none());

        // Windows direct command echo format: [0xD2 (210), battery_byte (18), 0, ...] -> 85%
        let res_windows_direct = [210u8, 18, 0, 49, 46, 52, 0, 0];
        let decoded = MouseManager::decode_aerox_prime_response(&res_windows_direct).expect("Should decode direct echo format");
        assert_eq!(decoded.0, Some(85));
        assert_eq!(decoded.1, false);
    }

    #[test]
    fn test_decode_aerox_prime_macos_format() {
        let res_85 = [0x15, 18];
        let decoded = MouseManager::decode_aerox_prime_response(&res_85).expect("Should decode");
        assert_eq!(decoded.0, Some(85));
        assert_eq!(decoded.1, false);

        let res_50_charging = [0x15, 11 | 0x80];
        let decoded = MouseManager::decode_aerox_prime_response(&res_50_charging).expect("Should decode");
        assert_eq!(decoded.0, Some(50));
        assert_eq!(decoded.1, true);
    }

    #[test]
    fn test_decode_rival3_650_windows_format() {
        let res_75 = [0x00, 75, 0, 0];
        let decoded = MouseManager::decode_rival3_650_response(&res_75).expect("Should decode");
        assert_eq!(decoded.0, Some(75));
        assert_eq!(decoded.1, false);

        let res_90_charging = [0x00, 90, 0, 1];
        let decoded = MouseManager::decode_rival3_650_response(&res_90_charging).expect("Should decode");
        assert_eq!(decoded.0, Some(90));
        assert_eq!(decoded.1, true);
    }

    #[test]
    fn test_decode_rival3_650_macos_format() {
        let res_75 = [75, 0, 0];
        let decoded = MouseManager::decode_rival3_650_response(&res_75).expect("Should decode");
        assert_eq!(decoded.0, Some(75));
        assert_eq!(decoded.1, false);

        let res_90_charging = [90, 0, 1];
        let decoded = MouseManager::decode_rival3_650_response(&res_90_charging).expect("Should decode");
        assert_eq!(decoded.0, Some(90));
        assert_eq!(decoded.1, true);
    }

    #[test]
    fn test_mock_mouse_manager_progression() {
        let mut manager = MouseManager::new(true);
        let info1 = manager.fetch_battery().expect("Mock battery should fetch");
        assert_eq!(info1.level, Some(84));
        assert_eq!(info1.is_charging, false);

        let info2 = manager.fetch_battery().expect("Mock battery should fetch");
        assert_eq!(info2.level, Some(83));
    }

    #[test]
    fn test_battery_tracker_charging_estimate() {
        let mut tracker = BatteryTracker::new();
        let t0 = Utc::now();
        let t1 = t0 + chrono::Duration::seconds(300); // 5 mins later

        tracker.add_sample_with_time(70, true, t0);
        tracker.add_sample_with_time(75, true, t1); // 5% gained in 300s -> 1% per min

        let estimate = tracker.estimate_time().expect("Should have estimate");
        assert_eq!(estimate, "Full in ~25m");
    }

    #[test]
    fn test_battery_tracker_discharging_estimate() {
        let mut tracker = BatteryTracker::new();
        let t0 = Utc::now();
        let t1 = t0 + chrono::Duration::seconds(3600); // 1 hour later

        tracker.add_sample_with_time(80, false, t0);
        tracker.add_sample_with_time(75, false, t1); // 5% lost in 1h -> 5% per hour -> 15 hours remaining for 75%

        let estimate = tracker.estimate_time().expect("Should have estimate");
        assert_eq!(estimate, "~15h 0m left");
    }
}
