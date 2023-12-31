## Steelseries Mouse Battery retrieval

This is a cool Windows app to retrieve the battery level of your Steelseries
mouse and display it in the system tray.

## Usage

Once started, you can hover over the icon to see the battery level. On right
click, you can exit the app or see the battery level again.

## Tested on

- Steelseries AEROX 3 Wireless (2.4G mode)
- Steelseries Prime Wireless
- Steelseries AEROX 9 Wireless (2.4G mode)

## Installation

0. Download the git repository as a zip file and extract it somewhere (or clone
   it with git)
1. Install Python 3 (https://www.python.org/downloads/) and make sure to check
   the box to add it to your PATH.
2. Pip install the following packages:
   - `pip install rivalcfg==4.9.1`
   - `pip install pystray==0.19.4`
   - `pip install pillow==10.0.0`
3. Run the script
   - Put the `start_mouse.bat` file as a shortcut in your startup folder
     (C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup).
   - You may need to change the bat script to point to your python installation
     if you have multiple versions installed or renamed the executable.
   - or you can run the script manually: `python3 mouse.py`

## Supported by the `rivalcfg` library

- [rivalcfg supported devices](https://flozz.github.io/rivalcfg/devices/index.html)

### Problems?

Firstly look at the `knownissues.md` file in the folder to see if your problem is listed.

Run the script in the `mouse-battery` folder:

```sh
python3 mouse.py
```

Get the output of the script and open an issue with the output and your mouse
model. Give extra detail, e.g. if you have multiple mice connected, your
connection mode etc.

## Uninstall guide
Sorry to see you go :(
1. Delete the `mouse-battery` folder
2. Delete the `start_mouse.bat` file in your startup folder
   (C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup)
3. Delete the python libraries installed via pip:
   - `pip uninstall rivalcfg`
   - `pip uninstall pystray`
   - `pip uninstall pillow`
4. Uninstall Python 3 if you don't need it anymore

### Thanks to

- [DeveloperX19](https://github.com/DeveloperX19) for the license of his
  intellectual property of the art of the icon.

## License

MIT: Feel free to use this code as you wish.
