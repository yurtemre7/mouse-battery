# SteelMouse ⚡

**SteelMouse** is a lightweight, cross-platform system tray application (Windows & macOS) that retrieves the battery level and charging status of your SteelSeries gaming mouse and displays it directly in your taskbar / menu bar.

![System Tray Image of SteelMouse](assets/image.png)

## 🌟 Key Features

- **⚡ Ultra-Lightweight & Fast**: Written natively in **Rust** (~2.1 MB executable, <10 MB RAM, <10 ms startup time).
- **🖥️ Cross-Platform**: Supports **Windows** (Win32 Tray) and **macOS** (Apple Silicon & Intel Menu Bar).
- **🎨 Dual Display Modes**:
  - **Hover Mode**: Clean battery graphic indicator in system tray with percentage tooltip.
  - **Icon Mode**: Dynamic percentage numbers drawn directly onto the system tray icon.
- **🔄 Configurable Refresh Interval**: Select refresh rate (1 min, 5 min, 10 min, 30 min, 1 hr) with instant zero-latency updates.
- **🖱️ 78+ Devices Supported**: Includes device configurations and USB HID battery parsing rules for Aerox, Prime, and Rival series.

---

## 💻 Tested & Working Devices

- SteelSeries Aerox 3 Wireless (Wired & 2.4G mode)
- SteelSeries Aerox 3 Gen 2 Wireless (Wired & 2.4G mode)
- SteelSeries Aerox 5 Wireless (Wired & 2.4G mode)
- SteelSeries Aerox 9 Wireless (Wired & 2.4G mode)
- SteelSeries Prime Wireless & Prime Mini Wireless
- SteelSeries Rival 3 Wireless (Gen 1 & Gen 2)
- SteelSeries Rival 650 Wireless

---

## 🚀 Installation

### Windows (Recommended)
1. Download the latest installer `SteelMouse_Rust_Setup.exe` from the [Releases](https://github.com/yurtemre7/steel-mouse/releases/) page.
2. Run the installer. It will install the application and add a shortcut to your Startup folder.

### macOS (Apple Silicon & Intel)
1. Download `steelmouse-macos-arm64` (Apple Silicon M1-M4) or `steelmouse-macos-x64` (Intel Mac) from [Releases](https://github.com/yurtemre7/steel-mouse/releases/).
2. Make it executable and run:
   ```bash
   chmod +x steelmouse-macos-arm64
   ./steelmouse-macos-arm64
   ```

---

## 🛠️ Building from Source

### Native Rust (Primary / Recommended)
Requires the [Rust toolchain](https://rustup.rs/):

```bash
# Clone the repository
git clone https://github.com/yurtemre7/steel-mouse.git
cd steel-mouse

# Run unit test suite (78+ devices & HID response decoders)
cargo test

# Run in mock testing mode
cargo run -- --mock

# Build production release binary (~2.1 MB)
cargo build --release
```

### Legacy Python Version (`steelmouse_python/`)
```bash
cd steelmouse_python

# Install dependencies
pip install -r requirements.txt

# Run legacy Python script
python mouse.py
```

---

## 🤝 Acknowledgements

- [DeveloperX19](https://github.com/DeveloperX19) for the icon art license.
- [flozz](https://github.com/flozz) for the `rivalcfg` library and reverse-engineered SteelSeries HID protocols.
- [Pyenb](https://github.com/Pyenb), [T-solidus-T](https://github.com/T-solidus-T), and [bossman90](https://github.com/bossman90) for contributions to the original Python codebase.

## 📄 License

MIT: Feel free to use this code as you wish. Mentioning the project is appreciated!
