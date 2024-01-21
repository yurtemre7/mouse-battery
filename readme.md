# Steelseries Mouse Battery Retrieval

This is a handy Windows application that retrieves the battery level of your Steelseries mouse and displays it in the system tray.

## Table of Contents

- [Usage](#usage)
- [Tested Devices](#tested-devices)
- [Installation](#installation)
  - [Latest Version (Recommended)](#latest-version-recommended)
- [Building from Source](#building-application-and-installer-from-source)
- [Supported Devices](#supported-devices)
- [Troubleshooting](#troubleshooting)
- [Uninstallation](#uninstallation)
- [Acknowledgements](#acknowledgements)
- [License](#license)

## Usage

Once the application is started, you can hover over the icon to see the battery level. Right-clicking allows you to exit the application or view the battery level again.

## Tested Devices

- Steelseries AEROX 3 Wireless (2.4G mode)
- Steelseries AEROX 9 Wireless (2.4G mode)
- Steelseries Prime Wireless

## Installation

### Latest Version (Recommended)

1. Download the latest application installer from the [Releases](https://github.com/yurtemre7/mouse-battery/releases/) tab.
2. Run the installer. This will install the application and place a shortcut in your Start Menu and add it to your auto-startup folder.
3. After installation, the installer will ask you if you want to run the application. If you choose not to, you can run it from the Start Menu shortcut or by restarting your computer.

#### Manual installation

1. Download the Git repository as a zip file and extract it somewhere (or clone it with Git).
2. Install Python 3 (<https://www.python.org/downloads/>) and ensure you check the box to add it to your PATH.
3. Install the following packages with Pip:
   - `pip install rivalcfg`
   - `pip install pystray`
   - `pip install pillow`
4. Run the script:
   - Place the `start_mouse.bat` file as a shortcut in your startup folder (C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup).
   - You may need to modify the bat script to point to your Python installation if you have multiple versions installed or have renamed the executable.
   - Alternatively, you can run the script manually.

## Building application and installer from source

To build the application yourself, check out [`building.md`](./building.md)

## Supported Devices

The `rivalcfg` library supports a variety of devices. A complete list of supported devices can be found [here](https://flozz.github.io/rivalcfg/devices/index.html).

## Troubleshooting

If you encounter any issues, first check the [`knownissues.md`](./knownissues.md) file in the repository to see if your problem is already listed.

If your problem is not listed, you can run the script in the `mouse-battery` folder. Note the output of the script and open an issue with the output and your mouse model. Provide additional details, such as whether you have multiple mice connected, your connection mode, etc.

```sh
python Steel_battery_status.py
```

## Uninstallation

### Latest Version

1. Go to the Control Panel and select "Uninstall a program".
2. Find "Steel_battery_status" in the list and click "Uninstall".

### Older Version

1. Delete the `mouse-battery` folder.
2. Delete the `start_mouse.bat` file in your startup folder (C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup).
3. Delete the Python libraries installed via Pip:
   - `pip uninstall rivalcfg`
   - `pip uninstall pystray`
   - `pip uninstall pillow`
4. Uninstall Python 3 if you no longer need it.

## Acknowledgements

- [DeveloperX19](https://github.com/DeveloperX19) for the license of his intellectual property of the icon art.
- [flozz](https://github.com/flozz) for the `rivalcfg` library and the idea of a standalone Python executable.

## License

MIT: Feel free to use this code as you wish. If you do use it, I'd appreciate a mention.
