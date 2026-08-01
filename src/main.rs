#![windows_subsystem = "windows"]

mod config;
mod devices;
mod gui;
mod hid;
mod icon;
mod log;
mod menu;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "SteelSeries Mouse Battery System Tray Monitor")]
struct Args {
    /// Enable mock mouse mode for testing/debugging
    #[arg(short, long, env = "MOCK_MOUSE")]
    mock: bool,

    /// Open native desktop GUI dashboard window on launch
    #[arg(short, long)]
    gui: bool,

    /// Print detailed HID diagnostic report of all connected SteelSeries devices
    #[arg(short, long)]
    dump_hid: bool,
}

fn load_app_icon() -> Option<eframe::egui::IconData> {
    let icon_bytes = include_bytes!("../assets/logo.png");
    if let Ok(img) = image::load_from_memory(icon_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        return Some(eframe::egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        });
    }
    None
}

fn main() {
    let args = Args::parse();

    if args.dump_hid {
        println!("=== SteelMouse HID Diagnostic Dump ===");
        if let Ok(api) = hidapi::HidApi::new() {
            let mut count = 0;
            for dev_info in api.device_list() {
                if dev_info.vendor_id() == 0x1038 {
                    count += 1;
                    println!(
                        "\n[{}] SteelSeries Device: PID=0x{:04x}, Interface={}, Path={:?}, Product={:?}",
                        count,
                        dev_info.product_id(),
                        dev_info.interface_number(),
                        dev_info.path(),
                        dev_info.product_string()
                    );
                    if let Ok(device) = dev_info.open_device(&api) {
                        for &cmd in &[0xD2u8, 0x92u8, 0xAAu8] {
                            let mut req = [0u8; 64];
                            req[0] = 0x00;
                            req[1] = cmd;
                            if cmd == 0xAA {
                                req[2] = 0x01;
                            }
                            let write_res = device.write(&req);
                            let mut res = [0u8; 64];
                            let read_res = device.read_timeout(&mut res, 300);

                            println!(
                                "  Cmd 0x{:02X}: write={:?}, read={:?}, bytes={:?}",
                                cmd,
                                write_res,
                                read_res.as_ref().map(|l| *l),
                                read_res.as_ref().map(|&l| &res[..l.min(16)]).unwrap_or(&[])
                            );
                        }
                    }
                }
            }
            if count == 0 {
                println!("No SteelSeries USB HID devices detected.");
            }
        }
        return;
    }

    log::init();
    log::log(&format!("Starting SteelMouse v2.1.0 (Rust) | mock={} gui={}", args.mock, args.gui));
    if args.mock {
        println!("Running in MOCK mode!");
    }

    let _ = ctrlc::set_handler(move || {
        log::log("Ctrl+C signal received. Exiting SteelMouse...");
        std::process::exit(0);
    });

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([440.0, 420.0])
        .with_resizable(false);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let mock_flag = args.mock;
    let start_hidden = !args.gui;

    let _ = eframe::run_native(
        "SteelMouse Dashboard",
        native_options,
        Box::new(move |cc| Ok(Box::new(gui::SteelMouseApp::new(cc, mock_flag, start_hidden)))),
    );
}
