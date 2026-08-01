//! # SteelMouse Rust Library
//!
//! High-performance native Rust protocol engine for SteelSeries gaming mice.
//! Supports 78 SteelSeries Product IDs, HID buffer draining, multi-pass 2.4GHz wireless querying,
//! charging status detection, and battery discharge rate estimation.

pub mod autostart;
pub mod config;
pub mod devices;
pub mod hid;
pub mod icon;
pub mod log;
pub mod menu;
pub mod protocol;

pub use protocol::{
    get_profile, BatteryInfo, BatteryKind, BatterySample, BatteryTracker, MouseManager, MouseProfile,
};
