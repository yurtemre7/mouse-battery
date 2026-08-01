#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod devices;
mod gui;
mod hid;
mod icon;
mod log;
mod menu;
pub mod protocol;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "SteelSeries Mouse Battery System Tray Monitor")]
struct Args {
    /// Enable mock mouse mode for testing/debugging
    #[arg(short, long, env = "MOCK_MOUSE")]
    mock: bool,

    /// Print detailed HID diagnostic report of all connected SteelSeries devices
    #[arg(short, long)]
    dump_hid: bool,

    /// Record live mouse HID responses and generate Rust test fixtures
    #[arg(short, long)]
    record_fixture: bool,
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

    if args.record_fixture {
        println!("=== SteelMouse Live HID Fixture Recorder ===");
        if let Ok(api) = hidapi::HidApi::new() {
            let mut count = 0;
            let mut captured_fixtures = Vec::new();

            for dev_info in api.device_list() {
                if dev_info.vendor_id() == 0x1038 {
                    count += 1;
                    let pid = dev_info.product_id();
                    let iface = dev_info.interface_number();
                    let prod = dev_info.product_string().unwrap_or("SteelSeries Device").to_string();

                    println!(
                        "\n[{}] Found Device: PID=0x{:04X}, Interface={}, Product='{}'",
                        count, pid, iface, prod
                    );

                    if let Ok(device) = dev_info.open_device(&api) {
                        for &cmd in &[0xD2u8, 0x92u8, 0xAAu8] {
                            // Drain buffer right before this command
                            let mut drain = [0u8; 64];
                            while device.read_timeout(&mut drain, 0).unwrap_or(0) > 0 {}

                            let mut req = [0u8; 64];
                            req[0] = 0x00;
                            req[1] = cmd;
                            if cmd == 0xAA {
                                req[2] = 0x01;
                            }

                            let write_ok = device.write(&[0x00, cmd]).is_ok()
                                || device.write(&req).is_ok()
                                || device.send_feature_report(&[0x00, cmd]).is_ok();

                            if write_ok {
                                for attempt in 0..5 {
                                    let mut res = [0u8; 64];
                                    match device.read_timeout(&mut res, 300) {
                                        Ok(n) if n >= 2 => {
                                            let bytes_vec = res[..n].to_vec();
                                            println!(
                                                "  Cmd 0x{:02X} attempt #{}: raw_bytes[0..8]={:?}",
                                                cmd, attempt, &bytes_vec[..n.min(8)]
                                            );
                                            captured_fixtures.push(serde_json::json!({
                                                "product_id": format!("0x{:04X}", pid),
                                                "interface": iface,
                                                "product_name": prod,
                                                "command": format!("0x{:02X}", cmd),
                                                "attempt": attempt,
                                                "raw_bytes": bytes_vec,
                                            }));
                                        }
                                        _ => break,
                                    }
                                }
                            }
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                    }
                }
            }

            if count == 0 {
                println!("⚠️ No SteelSeries USB HID devices detected on this system.");
            } else {
                let json_str = serde_json::to_string_pretty(&captured_fixtures).unwrap_or_default();
                let fixture_path = "fixtures/captured_mouse.json";
                let _ = std::fs::create_dir_all("fixtures");
                if std::fs::write(fixture_path, &json_str).is_ok() {
                    println!("\n✅ Captured {} HID response frames!", captured_fixtures.len());
                    println!("📁 Saved fixture to: {}", fixture_path);
                }

                println!("\n--- Ready-to-paste Rust Test Fixtures ---");
                for (idx, item) in captured_fixtures.iter().enumerate() {
                    if let Some(bytes) = item["raw_bytes"].as_array() {
                        let byte_vals: Vec<String> = bytes.iter().map(|b| b.to_string()).collect();
                        println!(
                            "    // Captured from {} (PID {}) Cmd {}",
                            item["product_name"].as_str().unwrap_or(""),
                            item["product_id"].as_str().unwrap_or(""),
                            item["command"].as_str().unwrap_or("")
                        );
                        println!(
                            "    let captured_sample_{} = [{}];",
                            idx,
                            byte_vals.join(", ")
                        );
                    }
                }
            }
        }
        return;
    }

    log::init();
    log::log(&format!("Starting SteelMouse v2.3.1 (Rust System Tray) | mock={}", args.mock));
    if args.mock {
        println!("Running in MOCK mode!");
    }

    let _ = ctrlc::set_handler(move || {
        log::log("Ctrl+C signal received. Exiting SteelMouse...");
        std::process::exit(0);
    });

    gui::run_tray_app(args.mock);
}
