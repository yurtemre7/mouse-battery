mod config;
mod devices;
mod hid;
mod icon;
mod menu;

use clap::Parser;
use chrono::Local;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{TrayIconBuilder, TrayIconEvent};
use muda::MenuEvent;

use config::{AppConfig, DisplayMode};
use hid::{BatteryInfo, MouseManager};
use icon::create_tray_icon;
use menu::TrayMenu;

#[derive(Parser, Debug)]
#[command(author, version, about = "SteelSeries Mouse Battery System Tray Monitor")]
struct Args {
    /// Enable mock mouse mode for testing/debugging
    #[arg(short, long, env = "MOCK_MOUSE")]
    mock: bool,
}

enum AppMessage {
    BatteryUpdated(Result<BatteryInfo, String>, String),
}

fn main() {
    let args = Args::parse();
    let mut config = AppConfig::load();

    println!("Starting SteelMouse v2.0.0 (Rust)...");
    if args.mock {
        println!("Running in MOCK mode!");
    }

    let event_loop = EventLoopBuilder::new().build();

    let (tx, rx): (Sender<AppMessage>, Receiver<AppMessage>) = channel();
    let tx_clone = tx.clone();

    // Signal channel to wake background thread instantly on interval/setting changes
    let (wake_tx, wake_rx): (Sender<()>, Receiver<()>) = channel();

    // Shared state between polling thread and main loop
    let current_config = Arc::new(Mutex::new(config.clone()));
    let config_polling = current_config.clone();

    // Spawn background HID battery polling thread
    let mock_flag = args.mock;
    thread::spawn(move || {
        let mut mouse_manager = MouseManager::new(mock_flag);

        loop {
            let result = mouse_manager.fetch_battery();
            let timestamp = Local::now().format("%H:%M:%S").to_string();

            let is_error = result.is_err();
            let _ = tx_clone.send(AppMessage::BatteryUpdated(result, timestamp));

            let sleep_secs = if is_error {
                12 // Fast retry on error (12s)
            } else {
                let cfg = config_polling.lock().unwrap();
                cfg.time_delta
            };

            // Wait in 1-second ticks or wake up immediately if wake signal is received
            let mut elapsed = 0u64;
            while elapsed < sleep_secs {
                if wake_rx.recv_timeout(Duration::from_secs(1)).is_ok() {
                    println!("Wake signal received! Polling battery immediately...");
                    break;
                }
                elapsed += 1;
                // Re-evaluate target in case time_delta was shortened
                let current_target = {
                    let cfg = config_polling.lock().unwrap();
                    cfg.time_delta
                };
                if elapsed >= current_target {
                    break;
                }
            }
        }
    });

    // Create initial tray icon & menu
    let initial_icon = create_tray_icon(None, false, config.display_mode);
    let tray_menu = TrayMenu::new(None, None, config.time_delta, config.display_mode);

    let mut tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu.menu.clone()))
            .with_tooltip("SteelMouse: Initializing...")
            .with_icon(initial_icon)
            .build()
            .expect("Failed to create system tray icon"),
    );

    let menu_event_receiver = MenuEvent::receiver();
    let tray_event_receiver = TrayIconEvent::receiver();

    let mut last_battery_info: Option<BatteryInfo> = None;
    let mut last_timestamp: Option<String> = None;

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + Duration::from_millis(100),
        );

        // Process incoming battery status messages
        while let Ok(AppMessage::BatteryUpdated(battery_res, timestamp)) = rx.try_recv() {
            let (info_opt, is_charging, level_opt) = match &battery_res {
                Ok(info) => {
                    println!("[{}] Battery update: {}", timestamp, info);
                    (Some(info.clone()), info.is_charging, info.level)
                }
                Err(err) => {
                    println!("[{}] Mouse query error: {}", timestamp, err);
                    (None, false, None)
                }
            };

            last_battery_info = info_opt.clone();
            last_timestamp = Some(timestamp.clone());

            // Update dynamic tray icon
            let new_icon = create_tray_icon(level_opt, is_charging, config.display_mode);
            if let Some(tray) = tray_icon.as_mut() {
                let _ = tray.set_icon(Some(new_icon));

                let tooltip = level_opt
                    .map(|l| format!("Battery: {}%", l))
                    .unwrap_or_else(|| "Battery: N/A".to_string());
                let _ = tray.set_tooltip(Some(tooltip));
            }

            // Update context menu
            tray_menu.update(
                info_opt.as_ref(),
                Some(&timestamp),
                config.time_delta,
                config.display_mode,
            );
        }

        // Process menu events
        while let Ok(event) = menu_event_receiver.try_recv() {
            if event.id == tray_menu.quit_item.id() {
                println!("Quit requested. Exiting...");
                tray_icon.take();
                *control_flow = ControlFlow::Exit;
                return;
            }

            if event.id == tray_menu.mode_hover_item.id() {
                config.display_mode = DisplayMode::Hover;
                config.save();
                *current_config.lock().unwrap() = config.clone();

                let level_opt = last_battery_info.as_ref().and_then(|b| b.level);
                let is_charging = last_battery_info.as_ref().map(|b| b.is_charging).unwrap_or(false);
                let new_icon = create_tray_icon(level_opt, is_charging, config.display_mode);
                if let Some(tray) = tray_icon.as_mut() {
                    let _ = tray.set_icon(Some(new_icon));
                }
                tray_menu.update(
                    last_battery_info.as_ref(),
                    last_timestamp.as_deref(),
                    config.time_delta,
                    config.display_mode,
                );
                let _ = wake_tx.send(());
            } else if event.id == tray_menu.mode_icon_item.id() {
                config.display_mode = DisplayMode::Icon;
                config.save();
                *current_config.lock().unwrap() = config.clone();

                let level_opt = last_battery_info.as_ref().and_then(|b| b.level);
                let is_charging = last_battery_info.as_ref().map(|b| b.is_charging).unwrap_or(false);
                let new_icon = create_tray_icon(level_opt, is_charging, config.display_mode);
                if let Some(tray) = tray_icon.as_mut() {
                    let _ = tray.set_icon(Some(new_icon));
                }
                tray_menu.update(
                    last_battery_info.as_ref(),
                    last_timestamp.as_deref(),
                    config.time_delta,
                    config.display_mode,
                );
                let _ = wake_tx.send(());
            }

            // Check interval menu items
            for (&seconds, item) in &tray_menu.interval_items {
                if event.id == item.id() {
                    println!("Changing interval to {} seconds ({:?})", seconds, item.id());
                    config.time_delta = seconds;
                    config.save();
                    *current_config.lock().unwrap() = config.clone();

                    tray_menu.update(
                        last_battery_info.as_ref(),
                        last_timestamp.as_deref(),
                        config.time_delta,
                        config.display_mode,
                    );
                    let _ = wake_tx.send(());
                    break;
                }
            }
        }

        // Drain tray icon click events
        while let Ok(_event) = tray_event_receiver.try_recv() {}
    });
}
