# Building the application and installer

This document provides instructions on how to build a standalone executable and an installer for the Steelseries Mouse Battery Retrieval application.

## Table of Contents

- [Creating a Standalone Executable](#1-creating-a-standalone-executable)
- [Creating an Installer with Inno Setup](#2-creating-an-installer-with-inno-setup)

## 1. Creating a Standalone Executable

To create a standalone executable for distribution, follow these steps:

1. Install PyInstaller, which we will use to build the executable. You can install it using pip:

   `pip install pyinstaller`

2. Build the standalone executable with PyInstaller. The --onefile option ensures the output is a single executable file, --noconsole prevents a console window from appearing when the program is run, and --add-data includes the necessary image files:

    `pyinstaller --onefile --noconsole --add-data images:images mouse.py`

## 2. Creating an Installer with Inno Setup

After creating the standalone executable, you can create an installer for the application using Inno Setup. Follow these steps:

1. **Download and Install Inno Setup**: You can download Inno Setup from the [official website](http://www.jrsoftware.org/isdl.php). After downloading, run the installer and follow the instructions to install Inno Setup on your computer.

2. **Open the Inno Setup Script**: Navigate to the `innosetup` folder in the project directory. Here, you will find a script file with the extension `.iss`. This is the Inno Setup script for your application. Open this script with Inno Setup.

3. **Compile the Installer**: With the script open in Inno Setup, click the "Compile" button. This will create the installer. The installer will be created in the output directory specified in the script (usually a subdirectory called `Output` in the `innosetup` folder).

Now you have an installer that you can distribute to users. They can run this installer to easily install your application on their system.

## 3. Creating the release

After creating the standalone executable and the installer, you can create a release for the application.
