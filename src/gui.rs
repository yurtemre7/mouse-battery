use eframe::egui;
use chrono::Local;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tray_icon::{TrayIcon, TrayIconBuilder};
use muda::MenuEvent;

use crate::config::{AppConfig, DisplayMode};
use crate::hid::{BatteryInfo, MouseManager};
use crate::icon::create_tray_icon;
use crate::log;
use crate::menu::TrayMenu;

enum AppMessage {
    BatteryUpdated(Result<BatteryInfo, String>, String),
}

// Shared flags set from the tray event thread, read by the eframe update() loop
struct TrayFlags {
    refresh_now: AtomicBool,
}

impl TrayFlags {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            refresh_now: AtomicBool::new(false),
        })
    }
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

pub struct SteelMouseApp {
    config: Arc<Mutex<AppConfig>>,
    battery_info: Option<BatteryInfo>,
    last_timestamp: Option<String>,
    status_msg: String,
    wake_tx: Sender<()>,
    rx: Receiver<AppMessage>,
    tray_icon: Option<TrayIcon>,
    tray_menu: Arc<Mutex<TrayMenu>>,
    flags: Arc<TrayFlags>,
}

impl SteelMouseApp {
    pub fn new(cc: &eframe::CreationContext<'_>, mock_mode: bool) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        log::log("SteelMouseApp::new() start (pure system tray mode)");

        let config = AppConfig::load();
        let current_config = Arc::new(Mutex::new(config.clone()));
        let config_for_hid = current_config.clone();

        let (tx, rx) = channel::<AppMessage>();
        let (wake_tx, wake_rx) = channel::<()>();

        let flags = TrayFlags::new();
        let egui_ctx = cc.egui_ctx.clone();

        // --- Thread 0: Ticker - forces update() to be called even when window is hidden ---
        {
            let ticker_ctx = egui_ctx.clone();
            thread::spawn(move || {
                log::log("ticker thread started");
                loop {
                    thread::sleep(Duration::from_millis(250));
                    ticker_ctx.request_repaint();
                }
            });
        }

        // --- Build tray icon & menu ---
        let initial_icon = create_tray_icon(None, false, config.display_mode);
        let tray_menu = Arc::new(Mutex::new(TrayMenu::new(
            None,
            None,
            config.time_delta,
            config.display_mode,
        )));

        let tray_icon = {
            let menu_lock = tray_menu.lock().unwrap();
            TrayIconBuilder::new()
                .with_menu(Box::new(menu_lock.menu.clone()))
                .with_tooltip("SteelMouse: Initializing...")
                .with_icon(initial_icon)
                .build()
                .ok()
        };

        // --- Thread 1: Tray menu event handler ---
        {
            let flags_tray = flags.clone();
            let egui_ctx_tray = egui_ctx.clone();
            let wake_tx_tray = wake_tx.clone();
            let menu_rx = MenuEvent::receiver();
            let tray_menu_ids = {
                let m = tray_menu.lock().unwrap();
                (
                    m.quit_item.id().clone(),
                    m.refresh_item.id().clone(),
                    m.mode_hover_item.id().clone(),
                    m.mode_icon_item.id().clone(),
                    m.interval_items.iter().map(|(&s, i)| (s, i.id().clone())).collect::<Vec<_>>(),
                )
            };
            let config_tray = current_config.clone();

            thread::spawn(move || {
                log::log("tray event thread started");
                let (quit_id, refresh_id, hover_id, icon_id, interval_ids) = tray_menu_ids;
                loop {
                    if let Ok(event) = menu_rx.recv() {
                        log::log(&format!("tray event: id={:?}", event.id));

                        if event.id == quit_id {
                            log::log("Quit clicked - calling exit(0)");
                            std::process::exit(0);
                        } else if event.id == refresh_id {
                            log::log("Refresh clicked");
                            flags_tray.refresh_now.store(true, Ordering::Relaxed);
                            let _ = wake_tx_tray.send(());
                            egui_ctx_tray.request_repaint();
                        } else {
                            let mut cfg = config_tray.lock().unwrap().clone();
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
                                *config_tray.lock().unwrap() = cfg;
                                let _ = wake_tx_tray.send(());
                                egui_ctx_tray.request_repaint();
                            }
                        }
                    }
                }
            });
        }

        // --- Thread 2: HID battery polling worker ---
        {
            let tx2 = tx;
            let egui_ctx2 = egui_ctx.clone();
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
                    egui_ctx2.request_repaint();
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

        #[cfg(target_os = "macos")]
        set_macos_activation_policy(true);

        cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));

        Self {
            config: current_config,
            battery_info: None,
            last_timestamp: None,
            status_msg: "Initializing...".to_string(),
            wake_tx,
            rx,
            tray_icon,
            tray_menu,
            flags,
        }
    }

    fn process_battery_messages(&mut self) {
        while let Ok(AppMessage::BatteryUpdated(battery_res, timestamp)) = self.rx.try_recv() {
            log::log(&format!("update: received BatteryUpdated at {}", timestamp));
            let (info_opt, is_charging, level_opt) = match &battery_res {
                Ok(info) => {
                    log::log(&format!("  -> Ok: name='{}' level={:?} charging={}", info.name, info.level, info.is_charging));
                    self.status_msg = format!("Updated at {}", timestamp);
                    (Some(info.clone()), info.is_charging, info.level)
                }
                Err(err) => {
                    log::log(&format!("  -> Err: {}", err));
                    self.status_msg = format!("Error: {}", err);
                    (None, false, None)
                }
            };

            self.battery_info = info_opt.clone();
            self.last_timestamp = Some(timestamp.clone());

            let cfg = self.config.lock().unwrap().clone();
            let new_icon = create_tray_icon(level_opt, is_charging, cfg.display_mode);

            if let Some(tray) = self.tray_icon.as_mut() {
                let _ = tray.set_icon(Some(new_icon));
                let tooltip = level_opt
                    .map(|l| format!("Battery: {}%", l))
                    .unwrap_or_else(|| "Battery: N/A".to_string());
                let _ = tray.set_tooltip(Some(tooltip));
            }

            let menu = self.tray_menu.lock().unwrap();
            menu.update(info_opt.as_ref(), Some(&timestamp), cfg.time_delta, cfg.display_mode);
        }
    }
}

impl eframe::App for SteelMouseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_battery_messages();

        if self.flags.refresh_now.swap(false, Ordering::Relaxed) {
            let _ = self.wake_tx.send(());
        }

        // Render nothing - pure background system tray app
        egui::CentralPanel::default().show(ctx, |_ui| {});
    }
}
