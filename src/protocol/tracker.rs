use std::collections::VecDeque;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct BatterySample {
    pub timestamp: DateTime<Utc>,
    pub level: u8,
    pub is_charging: bool,
}

/// Tracks historical battery samples to calculate charging/discharging time remaining.
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
