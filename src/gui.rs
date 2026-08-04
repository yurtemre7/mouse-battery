use chrono::Local;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::TrayIconBuilder;
use muda::MenuEvent;

use crate::config::{AppConfig, DisplayMode};
use crate::hid::{BatteryInfo, MouseManager};
use crate::icon::create_tray_icon;
use crate::log;
use crate::menu::TrayMenu;

enum AppMessage {
    BatteryUpdated(Result<BatteryInfo, String>, String),
}

#[cfg(target_os = "macos")]
fn set_macos_activation_policy(accessory: bool) {
    if let Some(mtm) = objc2_foundation::MainThreadMarker::new() {
        use objc2::ClassType;
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSImage};
        use objc2_foundation::NSData;

        let app = NSApplication::sharedApplication(mtm);
        let policy = if accessory {
            NSApplicationActivationPolicy::Accessory
        } else {
            NSApplicationActivationPolicy::Regular
        };
        let _ = app.setActivationPolicy(policy);

        let icon_bytes = include_bytes!("../assets/logo.png");
        let ns_data = NSData::with_bytes(icon_bytes);
        if let Some(ns_image) = NSImage::initWithData(NSImage::alloc(), &ns_data) {
            unsafe {
                app.setApplicationIconImage(Some(&ns_image));
            }
        }
    }
}

pub fn run_tray_app(mock_mode: bool) {
    log::log("run_tray_app: starting native tao event loop");

    #[cfg(target_os = "macos")]
    set_macos_activation_policy(true);

    let event_loop = EventLoopBuilder::new().build();

    let config = AppConfig::load();
    let current_config = Arc::new(Mutex::new(config.clone()));
    let config_for_hid = current_config.clone();

    let (tx, rx) = channel::<AppMessage>();
    let (wake_tx, wake_rx) = channel::<()>();

    // --- Build tray icon & menu ---
    let initial_icon = create_tray_icon(None, false, config.display_mode);
    let tray_menu = Arc::new(Mutex::new(TrayMenu::new(
        None,
        None,
        config.time_delta,
        config.display_mode,
    )));

    let mut tray_icon = {
        let menu_lock = tray_menu.lock().unwrap();
        TrayIconBuilder::new()
            .with_menu(Box::new(menu_lock.menu.clone()))
            .with_tooltip("SteelMouse: Initializing...")
            .with_icon(initial_icon)
            .build()
            .ok()
    };

    // --- Thread: HID battery polling worker ---
    {
        let tx2 = tx;
        thread::spawn(move || {
            log::log("HID worker thread started");
            let mut mouse_manager = MouseManager::new(mock_mode);
            loop {
                log::log("HID worker: calling fetch_battery");
                let result = mouse_manager.fetch_battery();
                log::log(&format!("HID worker: result={}", if result.is_ok() { "Ok" } else { "Err" }));
                let timestamp = Local::now().format("%H:%M:%S").to_string();
                let is_error = result.is_err();
                let _ = tx2.send(AppMessage::BatteryUpdated(result, timestamp));
                log::log("HID worker: message sent, sleeping");

                let sleep_secs = if is_error {
                    12
                } else {
                    config_for_hid.lock().unwrap().time_delta
                };

                let mut elapsed = 0u64;
                while elapsed < sleep_secs {
                    if wake_rx.recv_timeout(Duration::from_secs(1)).is_ok() {
                        break;
                    }
                    elapsed += 1;
                    if elapsed >= config_for_hid.lock().unwrap().time_delta {
                        break;
                    }
                }
            }
        });
    }

    let menu_rx = MenuEvent::receiver();
    let tray_menu_ids = {
        let m = tray_menu.lock().unwrap();
        (
            m.quit_item.id().clone(),
            m.refresh_item.id().clone(),
            m.mode_hover_item.id().clone(),
            m.mode_icon_item.id().clone(),
            m.autostart_item.id().clone(),
            m.export_diag_item.id().clone(),
            m.interval_items.iter().map(|(&s, i)| (s, i.id().clone())).collect::<Vec<_>>(),
        )
    };

    let (quit_id, refresh_id, hover_id, icon_id, autostart_id, export_diag_id, interval_ids) = tray_menu_ids;

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + Duration::from_millis(100),
        );

        // 1. Process incoming battery updates from HID worker
        while let Ok(AppMessage::BatteryUpdated(battery_res, timestamp)) = rx.try_recv() {
            log::log(&format!("tray loop: received BatteryUpdated at {}", timestamp));
            let (info_opt, is_charging, level_opt) = match &battery_res {
                Ok(info) => {
                    log::log(&format!("  -> Ok: name='{}' level={:?} charging={}", info.name, info.level, info.is_charging));
                    (Some(info.clone()), info.is_charging, info.level)
                }
                Err(err) => {
                    log::log(&format!("  -> Err: {}", err));
                    (None, false, None)
                }
            };

            let cfg = current_config.lock().unwrap().clone();
            let new_icon = create_tray_icon(level_opt, is_charging, cfg.display_mode);

            if let Some(tray) = tray_icon.as_mut() {
                let _ = tray.set_icon(Some(new_icon));
                let tooltip = level_opt
                    .map(|l| format!("Battery: {}%", l))
                    .unwrap_or_else(|| "SteelMouse: N/A".to_string());
                let _ = tray.set_tooltip(Some(tooltip));
            }

            let menu = tray_menu.lock().unwrap();
            menu.update(info_opt.as_ref(), Some(&timestamp), cfg.time_delta, cfg.display_mode);
        }

        // 2. Process system tray menu click events
        while let Ok(event) = menu_rx.try_recv() {
            log::log(&format!("tray event: id={:?}", event.id));
            if event.id == quit_id {
                log::log("Quit clicked - calling exit(0)");
                std::process::exit(0);
            } else if event.id == refresh_id {
                log::log("Refresh clicked");
                let _ = wake_tx.send(());
            } else if event.id == autostart_id {
                let m = tray_menu.lock().unwrap();
                let is_checked = m.autostart_item.is_checked();
                let target_state = !is_checked;
                log::log(&format!("Autostart toggled -> {}", target_state));
                let _ = crate::autostart::set_autostart(target_state);
                m.autostart_item.set_checked(target_state);
            } else if event.id == export_diag_id {
                log::log("Export Diagnostics clicked");
                std::thread::spawn(|| {
                    match crate::protocol::recorder::save_diagnostic_file(None) {
                        Ok(p) => {
                            log::log(&format!("Exported diagnostic report to {}", p.display()));
                            crate::protocol::recorder::open_file_in_file_manager(&p);
                        }
                        Err(e) => log::log(&format!("Failed to export diagnostic report: {}", e)),
                    }
                });
            } else {
                let mut cfg = current_config.lock().unwrap().clone();
                let mut changed = false;
                if event.id == hover_id {
                    cfg.display_mode = DisplayMode::Hover;
                    changed = true;
                } else if event.id == icon_id {
                    cfg.display_mode = DisplayMode::Icon;
                    changed = true;
                } else {
                    for (secs, id) in &interval_ids {
                        if event.id == *id {
                            cfg.time_delta = *secs;
                            changed = true;
                            break;
                        }
                    }
                }
                if changed {
                    log::log(&format!("Config changed: mode={:?} interval={}s", cfg.display_mode, cfg.time_delta));
                    cfg.save();
                    *current_config.lock().unwrap() = cfg;
                    let _ = wake_tx.send(());
                }
            }
        }
    });
}
