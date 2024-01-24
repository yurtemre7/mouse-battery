[Setup]
AppName=Steelseries Mouse Battery Retrieval
AppVersion=1.0
DefaultDirName={commonpf}\Steelseries Mouse Battery Retrieval
DefaultGroupName=Steelseries Mouse Battery Retrieval
UninstallDisplayIcon={app}\mouse.exe
OutputDir=.
OutputBaseFilename=SMBR_setup   
Compression=lzma
SolidCompression=yes

[Files]
Source: "..\dist\mouse.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Steelseries Mouse Battery Retrieval"; Filename: "{app}\mouse.exe"
Name: "{group}\Uninstall Steelseries Mouse Battery Retrieval"; Filename: "{uninstallexe}"
Name: "{commonstartup}\Steelseries Mouse Battery Retrieval"; Filename: "{app}\mouse.exe"; Tasks: autostart

[Run]
Filename: "{app}\mouse.exe"; Description: "Launch the application"; Flags: nowait postinstall skipifsilent

[Tasks]
Name: "autostart"; Description: "Start the application when Windows starts"; GroupDescription: "Additional tasks"; Flags: checkedonce