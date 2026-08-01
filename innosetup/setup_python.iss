#define MyAppName "SteelMouse (Python)"

[Setup]
AppName={#MyAppName}
AppVersion=2.1.0
DefaultDirName={commonpf}\{#MyAppName}
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\mouse.exe
OutputDir=.
OutputBaseFilename=SteelMouse_Python_Setup
Compression=lzma
SolidCompression=yes
SetupIconFile=..\steelmouse_python\images\logo.ico

[Files]
Source: "..\steelmouse_python\dist\mouse.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\mouse.exe"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{commonstartup}\{#MyAppName}"; Filename: "{app}\mouse.exe"; Tasks: autostart

[Run]
Filename: "{app}\mouse.exe"; Description: "Launch the application"; Flags: nowait postinstall skipifsilent

[Tasks]
Name: "autostart"; Description: "Start the application when Windows starts"; GroupDescription: "Additional tasks"; Flags: checkedonce
