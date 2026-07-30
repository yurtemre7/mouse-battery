# SteelMouse (Rust Edition)

A high-performance, native cross-platform (Windows & macOS & Linux) system tray application that displays the battery level and status of SteelSeries mice.

## Features

- **Ultra-Fast & Lightweight**: ~3 MB binary size, <10 MB RAM usage, <10 ms startup time.
- **Zero Runtime Dependencies**: Native machine code compiled directly for Windows (`Win32`), macOS (`AppKit`/`IOKit`), and Linux.
- **Dual Display Modes**:
  - **Hover Mode**: Clean battery fill graphic on tray icon.
  - **Icon Mode**: Dynamic percentage number text drawn directly on the system tray icon.
- **Live Menu Details**: Displays device name, battery %, charging status, and configurable refresh interval (1 min, 5 min, 10 min, 30 min, 1 hour).
- **Mock Mode**: Include `--mock` or set `MOCK_MOUSE=1` to test the UI and tray menu without physical hardware connected.

---

## Building Locally

### Requirements
- [Rust Toolchain](https://rustup.rs/) (1.80+)

### Commands

```bash
# Clone and enter directory
cd steelmouse_rust

# Run in mock mode for testing
cargo run -- --mock

# Build production release binary
cargo build --release
```

The release binary will be placed at `target/release/steelmouse` (or `steelmouse.exe` on Windows).

---

## CI / GitHub Actions

The repository includes a workflow in `.github/workflows/rust-build.yml` that builds:
- Windows Executable & Inno Setup Installer (`SteelMouse_Setup.exe`)
- macOS Apple Silicon & Intel Universal `.dmg` / `.app`
