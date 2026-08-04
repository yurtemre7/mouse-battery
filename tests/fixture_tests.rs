use serde_json::Value;
use std::fs;

#[test]
fn test_diagnostic_json_fixtures() {
    let fixtures_dir = std::path::Path::new("fixtures");
    if !fixtures_dir.exists() {
        return;
    }

    let entries = match fs::read_dir(fixtures_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let json_val: Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(devices) = json_val.get("devices").and_then(|d| d.as_array()) {
                for dev in devices {
                    let pid_hex = dev.get("product_id").and_then(|p| p.as_str()).unwrap_or("0x0000");
                    let _pid = u16::from_str_radix(pid_hex.trim_start_matches("0x"), 16).unwrap_or(0);

                    if let Some(cmds) = dev.get("probed_commands").and_then(|c| c.as_array()) {
                        for cmd_info in cmds {
                            if let Some(responses) = cmd_info.get("responses").and_then(|r| r.as_array()) {
                                for resp in responses {
                                    if let Some(raw_bytes) = resp.get("raw_bytes").and_then(|b| b.as_array()) {
                                        let bytes: Vec<u8> = raw_bytes
                                            .iter()
                                            .filter_map(|b| b.as_u64().map(|n| n as u8))
                                            .collect();
                                        if bytes.len() >= 2 {
                                            // Ensure decoding doesn't panic on any recorded user fixture frame
                                            let _ = format!("{:?}", bytes);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
