use eframe::egui;
use chrono::Local;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use muda::{MenuEvent, MenuEventReceiver};

use crate::config::{AppConfig, DisplayMode};
use crate::hid::{BatteryInfo, MouseManager};
use crate::icon::create_tray_icon;
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

pub struct SteelMouseApp {
    config: Arc<Mutex<AppConfig>>,
    battery_info: Option<BatteryInfo>,
    last_timestamp: Option<String>,
    status_msg: String,
    wake_tx: Sender<()>,
    rx: Receiver<AppMessage>,
    tray_icon: Option<TrayIcon>,
    tray_menu: TrayMenu,
    menu_receiver: &'static MenuEventReceiver,
    window_visible: bool,
}

impl SteelMouseApp {
    pub fn new(cc: &eframe::CreationContext<'_>, mock_mode: bool, start_hidden: bool) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let config = AppConfig::load();
        let current_config = Arc::new(Mutex::new(config.clone()));
        let config_polling = current_config.clone();

        let (tx, rx): (Sender<AppMessage>, Receiver<AppMessage>) = channel();
        let (wake_tx, wake_rx): (Sender<()>, Receiver<()>) = channel();

        let egui_ctx = cc.egui_ctx.clone();

        // Spawn background polling worker thread (All HID USB operations stay off the main GUI thread)
        thread::spawn(move || {
            let mut mouse_manager = MouseManager::new(mock_mode);

            loop {
                let result = mouse_manager.fetch_battery();
                let timestamp = Local::now().format("%H:%M:%S").to_string();

                let is_error = result.is_err();
                let _ = tx.send(AppMessage::BatteryUpdated(result, timestamp));
                egui_ctx.request_repaint();

                let sleep_secs = if is_error {
                    12
                } else {
                    let cfg = config_polling.lock().unwrap();
                    cfg.time_delta
                };

                let mut elapsed = 0u64;
                while elapsed < sleep_secs {
                    if wake_rx.recv_timeout(Duration::from_secs(1)).is_ok() {
                        break;
                    }
                    elapsed += 1;
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

        let initial_icon = create_tray_icon(None, false, config.display_mode);
        let tray_menu = TrayMenu::new(
            None,
            None,
            config.time_delta,
            config.display_mode,
        );

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu.menu.clone()))
            .with_tooltip("SteelMouse: Initializing...")
            .with_icon(initial_icon)
            .build()
            .ok();

        let menu_receiver = MenuEvent::receiver();

        if start_hidden {
            #[cfg(target_os = "macos")]
            set_macos_activation_policy(true);

            cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        Self {
            config: current_config,
            battery_info: None,
            last_timestamp: None,
            status_msg: "Initializing...".to_string(),
            wake_tx,
            rx,
            tray_icon,
            tray_menu,
            menu_receiver,
            window_visible: !start_hidden,
        }
    }

    fn poll_events(&mut self, ctx: &egui::Context) {
        // 1. Process incoming battery updates from background worker thread
        while let Ok(AppMessage::BatteryUpdated(battery_res, timestamp)) = self.rx.try_recv() {
            let (info_opt, is_charging, level_opt) = match &battery_res {
                Ok(info) => {
                    self.status_msg = format!("Updated at {}", timestamp);
                    (Some(info.clone()), info.is_charging, info.level)
                }
                Err(err) => {
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

            self.tray_menu.update(
                info_opt.as_ref(),
                Some(&timestamp),
                cfg.time_delta,
                cfg.display_mode,
            );
        }

        // 2. Process system tray menu clicks
        while let Ok(event) = self.menu_receiver.try_recv() {
            if event.id == self.tray_menu.quit_item.id() {
                println!("Quit requested. Exiting SteelMouse...");
                self.tray_icon.take();
                std::process::exit(0);
            }

            if event.id == self.tray_menu.dashboard_item.id() {
                self.window_visible = true;
                #[cfg(target_os = "macos")]
                set_macos_activation_policy(false);

                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                ctx.request_repaint();
            } else if event.id == self.tray_menu.refresh_item.id() {
                let _ = self.wake_tx.send(());
            } else {
                let mut cfg = self.config.lock().unwrap().clone();
                let mut changed = false;

                if event.id == self.tray_menu.mode_hover_item.id() {
                    cfg.display_mode = DisplayMode::Hover;
                    changed = true;
                } else if event.id == self.tray_menu.mode_icon_item.id() {
                    cfg.display_mode = DisplayMode::Icon;
                    changed = true;
                }

                for (&seconds, item) in &self.tray_menu.interval_items {
                    if event.id == item.id() {
                        cfg.time_delta = seconds;
                        changed = true;
                        break;
                    }
                }

                if changed {
                    cfg.save();
                    *self.config.lock().unwrap() = cfg.clone();

                    let level_opt = self.battery_info.as_ref().and_then(|b| b.level);
                    let is_charging = self.battery_info.as_ref().map(|b| b.is_charging).unwrap_or(false);
                    let new_icon = create_tray_icon(level_opt, is_charging, cfg.display_mode);
                    if let Some(tray) = self.tray_icon.as_mut() {
                        let _ = tray.set_icon(Some(new_icon));
                    }
                    self.tray_menu.update(
                        self.battery_info.as_ref(),
                        self.last_timestamp.as_deref(),
                        cfg.time_delta,
                        cfg.display_mode,
                    );
                    let _ = self.wake_tx.send(());
                }
            }
        }

        // Drain unused tray icon events
        let tray_receiver = TrayIconEvent::receiver();
        while let Ok(_event) = tray_receiver.try_recv() {}
    }
}

impl eframe::App for SteelMouseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_events(ctx);

        // Handle window close (X button) -> Hide to system tray instead of exiting
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.window_visible = false;
            #[cfg(target_os = "macos")]
            set_macos_activation_policy(true);

            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        ctx.request_repaint_after(Duration::from_secs(1));

        if !self.window_visible {
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);
                ui.heading("⚡ SteelMouse Dashboard");
                ui.label(egui::RichText::new("SteelSeries Battery & Device Monitor").small().color(egui::Color32::GRAY));
                ui.add_space(8.0);
            });

            // --- 1. Main Device Status Card ---
            ui.group(|ui| {
                ui.style_mut().spacing.item_spacing.y = 6.0;

                if let Some(info) = &self.battery_info {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new(&info.name).strong().size(18.0).color(egui::Color32::WHITE));
                        ui.add_space(4.0);

                        if let Some(lvl) = info.level {
                            let level_color = if info.is_charging {
                                egui::Color32::from_rgb(76, 217, 100)
                            } else if lvl > 30 {
                                egui::Color32::from_rgb(90, 200, 250)
                            } else if lvl > 15 {
                                egui::Color32::from_rgb(255, 204, 0)
                            } else {
                                egui::Color32::from_rgb(255, 59, 48)
                            };

                            ui.label(
                                egui::RichText::new(format!("{}%", lvl))
                                    .size(46.0)
                                    .strong()
                                    .color(level_color),
                            );

                            let progress = (lvl as f32) / 100.0;
                            ui.add(
                                egui::ProgressBar::new(progress)
                                    .text(format!("{}%", lvl))
                                    .animate(info.is_charging),
                            );
                        } else {
                            ui.label(egui::RichText::new("Battery: N/A").size(24.0).color(egui::Color32::GRAY));
                        }

                        ui.add_space(4.0);

                        let status_text = if info.is_charging {
                            "⚡ Status: Charging"
                        } else {
                            "🔋 Status: Discharging"
                        };
                        ui.label(egui::RichText::new(status_text).size(14.0).color(egui::Color32::LIGHT_GRAY));

                        if let Some(est) = &info.estimated_time {
                            ui.label(egui::RichText::new(est).size(13.0).italics().color(egui::Color32::LIGHT_BLUE));
                        }
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("⚠️ No supported SteelSeries mouse detected").size(15.0).color(egui::Color32::LIGHT_RED));
                        ui.label(egui::RichText::new("Please plug in your mouse or dongle.").small().color(egui::Color32::GRAY));
                        ui.add_space(10.0);
                    });
                }
            });

            ui.add_space(10.0);

            // --- 2. Interactive Settings & Controls ---
            ui.group(|ui| {
                ui.heading("⚙ Settings & Preferences");
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    if ui.button("🔄 Refresh Now").clicked() {
                        let _ = self.wake_tx.send(());
                    }
                    ui.label(egui::RichText::new(&self.status_msg).small().color(egui::Color32::GRAY));
                });

                ui.separator();

                let mut cfg = self.config.lock().unwrap().clone();
                let mut changed = false;

                ui.label(egui::RichText::new("System Tray Display Mode:").strong());
                if ui.radio_value(&mut cfg.display_mode, DisplayMode::Hover, "Hover tooltip for percentage").changed() {
                    changed = true;
                }
                if ui.radio_value(&mut cfg.display_mode, DisplayMode::Icon, "Render percentage number overlay on icon").changed() {
                    changed = true;
                }

                ui.separator();

                ui.label(egui::RichText::new("Background Refresh Interval:").strong());
                ui.horizontal(|ui| {
                    if ui.selectable_value(&mut cfg.time_delta, 60, "1m").changed() { changed = true; }
                    if ui.selectable_value(&mut cfg.time_delta, 300, "5m").changed() { changed = true; }
                    if ui.selectable_value(&mut cfg.time_delta, 600, "10m").changed() { changed = true; }
                    if ui.selectable_value(&mut cfg.time_delta, 1800, "30m").changed() { changed = true; }
                    if ui.selectable_value(&mut cfg.time_delta, 3600, "1h").changed() { changed = true; }
                });

                if changed {
                    cfg.save();
                    *self.config.lock().unwrap() = cfg;
                    let _ = self.wake_tx.send(());
                }
            });

            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("SteelMouse v2.1.0 • 78 SteelSeries Product IDs Supported").small().color(egui::Color32::DARK_GRAY));
            });
        });
    }
}
