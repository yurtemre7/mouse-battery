# ⚡ SteelMouse Changelog

> **Current Version:** `v2.3.1`  
> All notable changes to SteelMouse are automatically documented in this file based on release tags and git commit history.

## [v2.3.1](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.3.1) - 2026-08-01

### 🚀 Features

- docs: add CHANGELOG.md and generator script ([`7f99ef4`](https://github.com/yurtemre7/steel-mouse/commit/7f99ef4))
### ⚡ Performance & Architecture

- ci: format RELEASE_NOTES.md to contain only release tag commit bullets ([`27158c5`](https://github.com/yurtemre7/steel-mouse/commit/27158c5))
- ci: automate CHANGELOG.md & RELEASE_NOTES.md generation in release workflow ([`39e5703`](https://github.com/yurtemre7/steel-mouse/commit/39e5703))
---

## [v2.3.0](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.3.0) - 2026-08-01

### 🚀 Features

- release: v2.3.0 (decoupled steelmouse::protocol engine into standalone Rust library & crate) ([`d9f20cd`](https://github.com/yurtemre7/steel-mouse/commit/d9f20cd))
---

## [v2.2.1](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.2.1) - 2026-08-01

### 🚀 Features

- release: v2.2.1 (multi-pass write & read retry loop for 2.4GHz wireless stability) ([`61a9d27`](https://github.com/yurtemre7/steel-mouse/commit/61a9d27))
### ⚡ Performance & Architecture

- ci: extract release tag dynamically from Cargo.toml version ([`c820ad0`](https://github.com/yurtemre7/steel-mouse/commit/c820ad0))
---

## [v2.2.0](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.2.0) - 2026-08-01

### 🚀 Features

- release: v2.2.0 (pure native tao system tray app without egui window) ([`14c54b8`](https://github.com/yurtemre7/steel-mouse/commit/14c54b8))
---

## [v2.1.4](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.1.4) - 2026-08-01

### 🚀 Features

- release: v2.1.4 (pure system tray monitor, removed GUI window) ([`520cf50`](https://github.com/yurtemre7/steel-mouse/commit/520cf50))
---

## [v2.1.3](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.1.3) - 2026-08-01

### 🚀 Features

- release: v2.1.3 ([`3a40dbd`](https://github.com/yurtemre7/steel-mouse/commit/3a40dbd))
- Add --record-fixture recorder flag & integrate live captured Aerox 3 Wireless hardware fixture into tests ([`c62245a`](https://github.com/yurtemre7/steel-mouse/commit/c62245a))
### 🐛 Bug Fixes

- Refine recorder buffer draining per-command & re-capture clean live mouse fixture ([`3b19a4b`](https://github.com/yurtemre7/steel-mouse/commit/3b19a4b))
---

## [v2.1.2](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.1.2) - 2026-08-01

### 🚀 Features

- release: v2.1.2 ([`d824c9e`](https://github.com/yurtemre7/steel-mouse/commit/d824c9e))
---

## [v2.1.1](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.1.1) - 2026-08-01

### 🚀 Features

- release: v2.1.1 ([`c7063e4`](https://github.com/yurtemre7/steel-mouse/commit/c7063e4))
- Add ticker thread + 10x10 off-screen hidden window + hidden tick diagnostic logs ([`5143e45`](https://github.com/yurtemre7/steel-mouse/commit/5143e45))
- Add file-based logging (steelmouse.log) to diagnose Windows startup/tray issues ([`82b97b2`](https://github.com/yurtemre7/steel-mouse/commit/82b97b2))
- Add continuous event loop ticker to prevent Windows hidden tray state event loop freezes ([`275290c`](https://github.com/yurtemre7/steel-mouse/commit/275290c))
- Add Ctrl+C (SIGINT) signal handler for instant terminal exit during cargo run ([`7cffd11`](https://github.com/yurtemre7/steel-mouse/commit/7cffd11))
### 🐛 Bug Fixes

- Fix Aerox/Prime HID packet matching: accept direct command echo [210, 18, ...] without requiring res[1]==0x00 ([`3fcb852`](https://github.com/yurtemre7/steel-mouse/commit/3fcb852))
- Fix alternating 85/100%: skip non-battery HID reports in read loop (check res[1]==0x00) ([`8a9bd1f`](https://github.com/yurtemre7/steel-mouse/commit/8a9bd1f))
- Fix Refresh Now returning 100%: drain HID buffer before query, remove stale double-query ([`aa053fd`](https://github.com/yurtemre7/steel-mouse/commit/aa053fd))
- Fix Windows tray: dedicated blocking tray event thread + off-screen window trick so update() always runs ([`833327f`](https://github.com/yurtemre7/steel-mouse/commit/833327f))
- Show Windows console in debug builds, hide only in release ([`26628a1`](https://github.com/yurtemre7/steel-mouse/commit/26628a1))
- Fix Windows dashboard un-minimize window command and refine 85% battery decoding for Windows HID driver headers ([`9f1b6c3`](https://github.com/yurtemre7/steel-mouse/commit/9f1b6c3))
### 🔧 Maintenance & Improvements

- Windows: hide taskbar icon in tray mode (WS_EX_TOOLWINDOW), focus window on dashboard open ([`17fcf66`](https://github.com/yurtemre7/steel-mouse/commit/17fcf66))
- Keep HID USB operations exclusively on background worker thread to prevent Win32 GUI thread deadlock ([`076fa25`](https://github.com/yurtemre7/steel-mouse/commit/076fa25))
---

## [v2.1.0](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.1.0) - 2026-08-01

### 🚀 Features

- release: v2.1.0 ([`ade39db`](https://github.com/yurtemre7/steel-mouse/commit/ade39db))
- Add unit test suite for mouse profiles and HID battery report decoders, plus CI cargo test step ([`d86db95`](https://github.com/yurtemre7/steel-mouse/commit/d86db95))
### 🐛 Bug Fixes

- Fix Windows hidden event loop pause and initial tray menu population ([`9949c7a`](https://github.com/yurtemre7/steel-mouse/commit/9949c7a))
### 🔧 Maintenance & Improvements

- Implement dynamic battery charge time and remaining battery depletion estimation (Issue #12) ([`0a71643`](https://github.com/yurtemre7/steel-mouse/commit/0a71643))
- Optimize release profile (strip symbols, panic abort, LTO) to eliminate AV ML false positives ([`69e2362`](https://github.com/yurtemre7/steel-mouse/commit/69e2362))
- Swap Rust to root directory and Python to steelmouse_python legacy directory ([`8f97dc7`](https://github.com/yurtemre7/steel-mouse/commit/8f97dc7))
---

## [v2.0.7](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.0.7) - 2026-08-01

### 🚀 Features

- release: v2.0.7 ([`f7c8f0e`](https://github.com/yurtemre7/steel-mouse/commit/f7c8f0e))
- Add SteelSeries Aerox 3 Gen 2 Wireless (PID 0x1890 2.4GHz & 0x1892 wired) ([`90aca4a`](https://github.com/yurtemre7/steel-mouse/commit/90aca4a))
### 🐛 Bug Fixes

- Fix 2.4GHz wireless battery timeout and write packet matching for Prime Mini / Aerox ([`564ce6d`](https://github.com/yurtemre7/steel-mouse/commit/564ce6d))
### 🔧 Maintenance & Improvements

- Align READ_TIMEOUT_MS with rivalcfg 200ms default for all mice ([`ec1a727`](https://github.com/yurtemre7/steel-mouse/commit/ec1a727))
---

## [v2.0.6](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.0.6) - 2026-07-30

### 🚀 Features

- release: v2.0.6 ([`11b9bad`](https://github.com/yurtemre7/steel-mouse/commit/11b9bad))
---

## [v2.0.5](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.0.5) - 2026-07-30

### 🚀 Features

- release: v2.0.5 ([`4e88d78`](https://github.com/yurtemre7/steel-mouse/commit/4e88d78))
- Add Swatinem/rust-cache to speed up Rust CI compilation times ([`0679b39`](https://github.com/yurtemre7/steel-mouse/commit/0679b39))
- release: v2.0.4 ([`668d651`](https://github.com/yurtemre7/steel-mouse/commit/668d651))
### 🔧 Maintenance & Improvements

- Bump GitHub Actions to latest major releases (checkout@v7, setup-python@v7, upload-artifact@v7, download-artifact@v8, action-gh-release@v3) ([`a1b63d9`](https://github.com/yurtemre7/steel-mouse/commit/a1b63d9))
- Suppress Windows command prompt / console window on launch ([`19754b3`](https://github.com/yurtemre7/steel-mouse/commit/19754b3))
---

## [v2.0.3](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.0.3) - 2026-07-30

### 🚀 Features

- Add concurrency cancel-in-progress to cancel outdated workflow runs ([`b5c5cee`](https://github.com/yurtemre7/steel-mouse/commit/b5c5cee))
- release: v2.0.3 ([`821810b`](https://github.com/yurtemre7/steel-mouse/commit/821810b))
- release: v2.0.2 ([`e0290cb`](https://github.com/yurtemre7/steel-mouse/commit/e0290cb))
### 🔧 Maintenance & Improvements

- Execute back-to-back battery query twice with 20ms delay ([`2874606`](https://github.com/yurtemre7/steel-mouse/commit/2874606))
- Optimize discovery speed with 50ms read timeout and fast Windows path parsing ([`aee80d0`](https://github.com/yurtemre7/steel-mouse/commit/aee80d0))
---

## [v2.0.2-fixWindowsHIDbatterybyteoffsetindex](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.0.2-fixWindowsHIDbatterybyteoffsetindex) - 2026-07-30

_No detailed commit log for this release._

---

## [v2.0.2](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.0.2) - 2026-07-30

### 🚀 Features

- Add concurrency cancel-in-progress to cancel outdated workflow runs ([`b5c5cee`](https://github.com/yurtemre7/steel-mouse/commit/b5c5cee))
- release: v2.0.3 ([`821810b`](https://github.com/yurtemre7/steel-mouse/commit/821810b))
- release: v2.0.2 ([`e0290cb`](https://github.com/yurtemre7/steel-mouse/commit/e0290cb))
- Fix HID detection logic and add --dump-hid diagnostic flag ([`2dec852`](https://github.com/yurtemre7/steel-mouse/commit/2dec852))
- release: v2.0.2 - fix Windows HID battery byte offset index ([`3939c71`](https://github.com/yurtemre7/steel-mouse/commit/3939c71))
- Configure CI to run only on commits starting with 'release:', git tags, or manual dispatch ([`0d827f2`](https://github.com/yurtemre7/steel-mouse/commit/0d827f2))
### 🔧 Maintenance & Improvements

- Execute back-to-back battery query twice with 20ms delay ([`2874606`](https://github.com/yurtemre7/steel-mouse/commit/2874606))
- Optimize discovery speed with 50ms read timeout and fast Windows path parsing ([`aee80d0`](https://github.com/yurtemre7/steel-mouse/commit/aee80d0))
- Update repository URLs to steel-mouse ([`2d455a0`](https://github.com/yurtemre7/steel-mouse/commit/2d455a0))
- Enhance battery response byte decoding for cross-platform report ID variations ([`ce9ac13`](https://github.com/yurtemre7/steel-mouse/commit/ce9ac13))
- Update download-artifact in build.yml with merge-multiple to attach all macOS and Windows binaries to releases ([`d337718`](https://github.com/yurtemre7/steel-mouse/commit/d337718))
---

## [v2.0.1](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.0.1) - 2026-07-30

### 🐛 Bug Fixes

- Fix Windows HID interface matching and report ID parsing for battery detection ([`28387f1`](https://github.com/yurtemre7/steel-mouse/commit/28387f1))
---

## [v2.0.0](https://github.com/yurtemre7/steel-mouse/releases/tag/v2.0.0) - 2026-07-30

### 🚀 Features

- Add native Rust implementation with 76+ rivalcfg mouse profiles, macOS support, and multi-build CI workflow ([`d958ea4`](https://github.com/yurtemre7/steel-mouse/commit/d958ea4))
---

## [v1.3.2](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.3.2) - 2026-06-15

### 🚀 Features

- feat: update github actions ([`e3b90c9`](https://github.com/yurtemre7/steel-mouse/commit/e3b90c9))
---

## [v1.3.1](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.3.1) - 2026-06-15

### 🚀 Features

- feat: update packages and migrate getdata to get_flattened_data ([`87bfaa8`](https://github.com/yurtemre7/steel-mouse/commit/87bfaa8))
- feat: remove running workflow on pull requests ([`a55d2e3`](https://github.com/yurtemre7/steel-mouse/commit/a55d2e3))
---

## [v1.3.0](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.3.0) - 2026-03-07

### 🚀 Features

- feat: add mock mouse class ([`ce37189`](https://github.com/yurtemre7/steel-mouse/commit/ce37189))
- Merge pull request #1 from Thrump/copilot/add-battery-percentage-display ([`f766a3c`](https://github.com/yurtemre7/steel-mouse/commit/f766a3c))
- feat: add tray battery percentage display toggle ([`ec96022`](https://github.com/yurtemre7/steel-mouse/commit/ec96022))
### 🐛 Bug Fixes

- fix: remove unused display mode configuration file ([`bafcb03`](https://github.com/yurtemre7/steel-mouse/commit/bafcb03))
- fix: improve 100 readability in tray icon mode by removing text border ([`d1ecabe`](https://github.com/yurtemre7/steel-mouse/commit/d1ecabe))
- fix: show number-only tray icon instead of text overlay on battery fill ([`8b85e19`](https://github.com/yurtemre7/steel-mouse/commit/8b85e19))
- fix: improve tray percentage text contrast ([`33f4423`](https://github.com/yurtemre7/steel-mouse/commit/33f4423))
### 📝 Documentation & Chores

- chore: update github action to only build on version changes ([`5d4978e`](https://github.com/yurtemre7/steel-mouse/commit/5d4978e))
- chore: set app version to 1.3.0 ([`f9bc67d`](https://github.com/yurtemre7/steel-mouse/commit/f9bc67d))
- chore: remove pycache artifact and ignore bytecode ([`8a24c7d`](https://github.com/yurtemre7/steel-mouse/commit/8a24c7d))
### 🔧 Maintenance & Improvements

- Merge pull request #15 from Thrump/feature/system-number ([`286d2f6`](https://github.com/yurtemre7/steel-mouse/commit/286d2f6))
- Tried to fit text within tray ([`c213064`](https://github.com/yurtemre7/steel-mouse/commit/c213064))
- Initial plan ([`5525597`](https://github.com/yurtemre7/steel-mouse/commit/5525597))
---

## [v1.2.4](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.2.4) - 2026-02-24

### 🚀 Features

- add building ([`4ec31e4`](https://github.com/yurtemre7/steel-mouse/commit/4ec31e4))
- add ([`537a171`](https://github.com/yurtemre7/steel-mouse/commit/537a171))
- add ([`bef6163`](https://github.com/yurtemre7/steel-mouse/commit/bef6163))
### 📝 Documentation & Chores

- chore: rivalcfg version to 4.16.0 ([`fe5567c`](https://github.com/yurtemre7/steel-mouse/commit/fe5567c))
### 🔧 Maintenance & Improvements

- update ([`988b0a1`](https://github.com/yurtemre7/steel-mouse/commit/988b0a1))
- update paths and action versions ([`f239199`](https://github.com/yurtemre7/steel-mouse/commit/f239199))
- update texts ([`86460af`](https://github.com/yurtemre7/steel-mouse/commit/86460af))
- adapt path ([`88e88a2`](https://github.com/yurtemre7/steel-mouse/commit/88e88a2))
---

## [v1.2.3](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.2.3) - 2025-12-28

### 🔧 Maintenance & Improvements

- update version ([`d5821c1`](https://github.com/yurtemre7/steel-mouse/commit/d5821c1))
---

## [v1.2.2](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.2.2) - 2025-12-28

### 🔧 Maintenance & Improvements

- init release with version ([`a49fab5`](https://github.com/yurtemre7/steel-mouse/commit/a49fab5))
- init ([`9deacc5`](https://github.com/yurtemre7/steel-mouse/commit/9deacc5))
- update website ([`a1f7b45`](https://github.com/yurtemre7/steel-mouse/commit/a1f7b45))
- update version ([`2dc970f`](https://github.com/yurtemre7/steel-mouse/commit/2dc970f))
---

## [v1.2.1](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.2.1) - 2025-08-14

### 🚀 Features

- add uv ([`d18b3fd`](https://github.com/yurtemre7/steel-mouse/commit/d18b3fd))
- add: include website link in the README ([`928f858`](https://github.com/yurtemre7/steel-mouse/commit/928f858))
- add initial HTML, CSS, and JavaScript files for Mouse Battery App ([`7b0dff3`](https://github.com/yurtemre7/steel-mouse/commit/7b0dff3))
- remove old HTML, CSS, and JavaScript files; add new documentation files for Mouse Battery App ([`2954257`](https://github.com/yurtemre7/steel-mouse/commit/2954257))
- add initial HTML, CSS, and JavaScript files for Mouse Battery App ([`7a24302`](https://github.com/yurtemre7/steel-mouse/commit/7a24302))
### 🐛 Bug Fixes

- fix: update rivalcfg version to 4.15.0 ([`64bc34e`](https://github.com/yurtemre7/steel-mouse/commit/64bc34e))
- fix: update installation instructions and file references in README ([`bda1ada`](https://github.com/yurtemre7/steel-mouse/commit/bda1ada))
- fix: update project name from "Steelseries Mouse Battery Retrieval" to "SteelMouse" in README ([`cbd4bc2`](https://github.com/yurtemre7/steel-mouse/commit/cbd4bc2))
- fix: update known issues solution for "pillow" package version ([`8ca0eae`](https://github.com/yurtemre7/steel-mouse/commit/8ca0eae))
- fix: update image paths in index.html for correct loading ([`a1e8525`](https://github.com/yurtemre7/steel-mouse/commit/a1e8525))
### 🔧 Maintenance & Improvements

- format code ([`5abe300`](https://github.com/yurtemre7/steel-mouse/commit/5abe300))
- remove HTML, CSS, and JavaScript files for Mouse Battery App ([`4b196ac`](https://github.com/yurtemre7/steel-mouse/commit/4b196ac))
---

## [v1.2.0](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.2.0) - 2025-05-26

### 🚀 Features

- add default 5 mins time_delta ([`fa328ac`](https://github.com/yurtemre7/steel-mouse/commit/fa328ac))
- add image of app to readme ([`e33d001`](https://github.com/yurtemre7/steel-mouse/commit/e33d001))
### ⚡ Performance & Architecture

- refactor time delta handling and improve menu functionality ([`d2a8e45`](https://github.com/yurtemre7/steel-mouse/commit/d2a8e45))
### 🔧 Maintenance & Improvements

- update version ([`be2e426`](https://github.com/yurtemre7/steel-mouse/commit/be2e426))
- update requirements to latest versions ([`1bba638`](https://github.com/yurtemre7/steel-mouse/commit/1bba638))
---

## [v1.1.2](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.1.2) - 2025-01-21

### 🚀 Features

- update version of rivalcfg to support new devices ([`0300ce1`](https://github.com/yurtemre7/steel-mouse/commit/0300ce1))
- add new tested device ([`eedf7e1`](https://github.com/yurtemre7/steel-mouse/commit/eedf7e1))
### 🔧 Maintenance & Improvements

- update version ([`c5ee04f`](https://github.com/yurtemre7/steel-mouse/commit/c5ee04f))
- update name ([`4045165`](https://github.com/yurtemre7/steel-mouse/commit/4045165))
---

## [v1.1.1](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.1.1) - 2024-06-07

### 🚀 Features

- add battery charging / dis-charging to tray icon and menu + reduce app refresh timer from 10min to 1min ([`a08a4ec`](https://github.com/yurtemre7/steel-mouse/commit/a08a4ec))
- add new tested device ([`1fdbb47`](https://github.com/yurtemre7/steel-mouse/commit/1fdbb47))
### 🔧 Maintenance & Improvements

- update readme acknowledgements ([`140e658`](https://github.com/yurtemre7/steel-mouse/commit/140e658))
- update version ([`745df93`](https://github.com/yurtemre7/steel-mouse/commit/745df93))
- update title ([`125698f`](https://github.com/yurtemre7/steel-mouse/commit/125698f))
- update version ([`bc57613`](https://github.com/yurtemre7/steel-mouse/commit/bc57613))
---

## [v1.1.0](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.1.0) - 2024-05-20

### 🔧 Maintenance & Improvements

- update rivalcfg package ([`da09c25`](https://github.com/yurtemre7/steel-mouse/commit/da09c25))
- change version ([`bebdb2c`](https://github.com/yurtemre7/steel-mouse/commit/bebdb2c))
---

## [v1.0.3](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.0.3) - 2024-03-08

### 🚀 Features

- add new contributor ([`eb49285`](https://github.com/yurtemre7/steel-mouse/commit/eb49285))
- Improve wording, add click to refresh & better event handling/sleeping & icon placement ([`dde6efb`](https://github.com/yurtemre7/steel-mouse/commit/dde6efb))
- add ([`66d2590`](https://github.com/yurtemre7/steel-mouse/commit/66d2590))
---

## [v1.0.2](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.0.2) - 2024-01-24

### 🚀 Features

- Add check for running mouse.exe process ([`be6ee58`](https://github.com/yurtemre7/steel-mouse/commit/be6ee58))
- add build script ([`d812086`](https://github.com/yurtemre7/steel-mouse/commit/d812086))
- add app icon ([`42f2518`](https://github.com/yurtemre7/steel-mouse/commit/42f2518))
- update req. versions. Added reqs-txt file and instructions ([`2c83336`](https://github.com/yurtemre7/steel-mouse/commit/2c83336))
- Huge update. Rewrote entire readme, building and knowissues. Edited the python code to be a little more structured. Added an Innosetup file to create and easy installer. (No more manual copying) ([`830d7fe`](https://github.com/yurtemre7/steel-mouse/commit/830d7fe))
### 🐛 Bug Fixes

- small gitignore fix ([`a7fdbd1`](https://github.com/yurtemre7/steel-mouse/commit/a7fdbd1))
### 🔧 Maintenance & Improvements

- lel this didnt work ([`42be9f8`](https://github.com/yurtemre7/steel-mouse/commit/42be9f8))
- update install instructions ([`5219c3e`](https://github.com/yurtemre7/steel-mouse/commit/5219c3e))
- gitignore ([`68899c5`](https://github.com/yurtemre7/steel-mouse/commit/68899c5))
- Merge pull request #3 from Pyenb/master ([`6366eb0`](https://github.com/yurtemre7/steel-mouse/commit/6366eb0))
- Merge branch 'master' into master ([`b542fb3`](https://github.com/yurtemre7/steel-mouse/commit/b542fb3))
- reolve requests ([`0625735`](https://github.com/yurtemre7/steel-mouse/commit/0625735))
- refrence files instead of links ([`7b9e899`](https://github.com/yurtemre7/steel-mouse/commit/7b9e899))
- delete spec ([`d856824`](https://github.com/yurtemre7/steel-mouse/commit/d856824))
---

## [v1.0.1](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.0.1) - 2024-01-24

### 🐛 Bug Fixes

- hotfix ([`906c2f4`](https://github.com/yurtemre7/steel-mouse/commit/906c2f4))
### 🔧 Maintenance & Improvements

- revert to old ([`69b2b1d`](https://github.com/yurtemre7/steel-mouse/commit/69b2b1d))
- close after thread ends ([`e4df58f`](https://github.com/yurtemre7/steel-mouse/commit/e4df58f))
---

## [v1.0.0](https://github.com/yurtemre7/steel-mouse/releases/tag/v1.0.0) - 2024-01-20

### 🚀 Features

- update for new release ([`deffdb7`](https://github.com/yurtemre7/steel-mouse/commit/deffdb7))
- add name and fix code a little bit for windows standalone ([`784ba34`](https://github.com/yurtemre7/steel-mouse/commit/784ba34))
- add gitignore for build stuff ([`e0c97d9`](https://github.com/yurtemre7/steel-mouse/commit/e0c97d9))
- add new debug ([`f4814d9`](https://github.com/yurtemre7/steel-mouse/commit/f4814d9))
- add remark to the knownissues.md file ([`9bf4e0a`](https://github.com/yurtemre7/steel-mouse/commit/9bf4e0a))
- add known issues ([`6364b30`](https://github.com/yurtemre7/steel-mouse/commit/6364b30))
- show new lib install ([`f170c75`](https://github.com/yurtemre7/steel-mouse/commit/f170c75))
- add versions for the packages and uninstall guide ([`5842fa5`](https://github.com/yurtemre7/steel-mouse/commit/5842fa5))
- add supported devices remark ([`79472bd`](https://github.com/yurtemre7/steel-mouse/commit/79472bd))
- add last update ([`d0da474`](https://github.com/yurtemre7/steel-mouse/commit/d0da474))
- add tested on, better installation guide and problems section ([`e3741a4`](https://github.com/yurtemre7/steel-mouse/commit/e3741a4))
- add windows to readme ([`0716689`](https://github.com/yurtemre7/steel-mouse/commit/0716689))
- add license text to readme ([`ade3bb0`](https://github.com/yurtemre7/steel-mouse/commit/ade3bb0))
- add usage section ([`ac66c2e`](https://github.com/yurtemre7/steel-mouse/commit/ac66c2e))
- add more code annotations ([`567804c`](https://github.com/yurtemre7/steel-mouse/commit/567804c))
- add PATH remark ([`413f82e`](https://github.com/yurtemre7/steel-mouse/commit/413f82e))
- add description of custom python executables ([`564c177`](https://github.com/yurtemre7/steel-mouse/commit/564c177))
- add MIT license ([`0ca0c19`](https://github.com/yurtemre7/steel-mouse/commit/0ca0c19))
### 🐛 Bug Fixes

- fix pillow 10 error ([`63df1de`](https://github.com/yurtemre7/steel-mouse/commit/63df1de))
### 🔧 Maintenance & Improvements

- update version in readme ([`d0477e8`](https://github.com/yurtemre7/steel-mouse/commit/d0477e8))
- make it try and catch so the script keeps on trying ([`cf1e6ce`](https://github.com/yurtemre7/steel-mouse/commit/cf1e6ce))
- update tested on list ([`74760cb`](https://github.com/yurtemre7/steel-mouse/commit/74760cb))
- check if mouse is None ([`62f24c8`](https://github.com/yurtemre7/steel-mouse/commit/62f24c8))
- init ([`735e9fe`](https://github.com/yurtemre7/steel-mouse/commit/735e9fe))
---

