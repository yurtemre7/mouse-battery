#define MyAppName "SteelMouse"

[Setup]
AppName={#MyAppName}
AppVersion=1.2.0
DefaultDirName={commonpf}\{#MyAppName}
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\mouse.exe
OutputDir=.
OutputBaseFilename=SteelMouse_Setup
Compression=lzma
SolidCompression=yes
SetupIconFile=..\images\logo.ico

[Files]
Source: "..\dist\mouse.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\mouse.exe"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{commonstartup}\{#MyAppName}"; Filename: "{app}\mouse.exe"; Tasks: autostart

[Run]
Filename: "{app}\mouse.exe"; Description: "Launch the application"; Flags: nowait postinstall skipifsilent

[Tasks]
Name: "autostart"; Description: "Start the application when Windows starts"; GroupDescription: "Additional tasks"; Flags: checkedonce

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\mouse.exe"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{commonstartup}\{#MyAppName}"; Filename: "{app}\mouse.exe"; Tasks: autostart