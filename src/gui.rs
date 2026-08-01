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
    open_dashboard: AtomicBool,
    refresh_now: AtomicBool,
}

impl TrayFlags {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            open_dashboard: AtomicBool::new(false),
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
    window_visible: bool,
}

impl SteelMouseApp {
    pub fn new(cc: &eframe::CreationContext<'_>, mock_mode: bool, start_hidden: bool) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        log::log("SteelMouseApp::new() start");

        let config = AppConfig::load();
        let current_config = Arc::new(Mutex::new(config.clone()));
        let config_for_hid = current_config.clone();

        let (tx, rx) = channel::<AppMessage>();
        let (wake_tx, wake_rx) = channel::<()>();

        let flags = TrayFlags::new();
        let egui_ctx = cc.egui_ctx.clone();

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

        // --- Thread 1: Tray menu event handler (independent of eframe update loop!) ---
        // This MUST run independently so Quit and Open Dashboard work even when
        // the eframe window is hidden and update() is not being called by winit.
        {
            let flags_tray = flags.clone();
            let egui_ctx_tray = egui_ctx.clone();
            let wake_tx_tray = wake_tx.clone();
            let menu_rx = MenuEvent::receiver();
            let tray_menu_ids = {
                let m = tray_menu.lock().unwrap();
                (
                    m.quit_item.id().clone(),
                    m.dashboard_item.id().clone(),
                    m.refresh_item.id().clone(),
                    m.mode_hover_item.id().clone(),
                    m.mode_icon_item.id().clone(),
                    m.interval_items.iter().map(|(&s, i)| (s, i.id().clone())).collect::<Vec<_>>(),
                )
            };
            let config_tray = current_config.clone();

            thread::spawn(move || {
                log::log("tray event thread started");
                let (quit_id, dashboard_id, refresh_id, hover_id, icon_id, interval_ids) = tray_menu_ids;
                loop {
                    // Block until a menu event arrives (no busy-spinning)
                    if let Ok(event) = menu_rx.recv() {
                        log::log(&format!("tray event: id={:?}", event.id));

                        if event.id == quit_id {
                            log::log("Quit clicked - calling exit(0)");
                            std::process::exit(0);
                        } else if event.id == dashboard_id {
                            log::log("Dashboard clicked - setting flag");
                            flags_tray.open_dashboard.store(true, Ordering::Relaxed);
                            egui_ctx_tray.request_repaint();
                        } else if event.id == refresh_id {
                            log::log("Refresh clicked");
                            flags_tray.refresh_now.store(true, Ordering::Relaxed);
                            let _ = wake_tx_tray.send(());
                            egui_ctx_tray.request_repaint();
                        } else {
                            // Check display mode / interval changes
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

        // On macOS: hide dock icon when in tray mode
        if start_hidden {
            #[cfg(target_os = "macos")]
            set_macos_activation_policy(true);
        }

        // On Windows: keep eframe window "visible" but off-screen so update() keeps firing.
        // On macOS: we can safely hide it via Visible(false) since Cocoa handles repaints differently.
        #[cfg(target_os = "windows")]
        if start_hidden {
            // Move far off-screen + make tiny so it doesn't appear on taskbar
            egui_ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition([30000.0, 30000.0].into()));
            egui_ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([1.0, 1.0].into()));
            egui_ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
        }

        #[cfg(not(target_os = "windows"))]
        if start_hidden {
            egui_ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
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
            flags,
            window_visible: !start_hidden,
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

    fn show_window(&mut self, ctx: &egui::Context) {
        log::log("show_window: making dashboard visible");
        self.window_visible = true;

        #[cfg(target_os = "macos")]
        set_macos_activation_policy(false);

        // Restore window position/size on Windows (we moved it off-screen)
        #[cfg(target_os = "windows")]
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([440.0, 420.0].into()));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition([100.0, 100.0].into()));
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        ctx.request_repaint();
    }

    fn hide_window(&mut self, ctx: &egui::Context) {
        log::log("hide_window: hiding dashboard to tray");
        self.window_visible = false;

        #[cfg(target_os = "macos")]
        set_macos_activation_policy(true);

        #[cfg(target_os = "windows")]
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([1.0, 1.0].into()));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition([30000.0, 30000.0].into()));
        }

        #[cfg(not(target_os = "windows"))]
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }
}

impl eframe::App for SteelMouseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Always process battery messages regardless of window visibility
        self.process_battery_messages();

        // Check flags set by the tray event thread
        if self.flags.open_dashboard.swap(false, Ordering::Relaxed) {
            self.show_window(ctx);
        }
        if self.flags.refresh_now.swap(false, Ordering::Relaxed) {
            let _ = self.wake_tx.send(());
        }

        // Handle window X button -> hide to tray
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.hide_window(ctx);
            return;
        }

        // Keep update() alive even when hidden on Windows (we're off-screen, not truly hidden)
        ctx.request_repaint_after(Duration::from_secs(1));

        if !self.window_visible {
            // Render nothing but keep eframe ticking
            egui::CentralPanel::default().show(ctx, |_ui| {});
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

                        let status_text = if info.is_charging { "⚡ Status: Charging" } else { "🔋 Status: Discharging" };
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

            // --- 2. Settings & Controls ---
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
