# OS Integration Setup

## Windows Context Menu

### Installation
1. Right-click `windows-context-menu.reg`
2. Click "Merge"
3. Confirm UAC prompt
4. Restart Explorer (or reboot)

**Result**: Right-click any file → "Compress with QRES"

### Uninstallation
1. Right-click `windows-uninstall-context-menu.reg`
2. Click "Merge"

---

## macOS Quick Action

### Installation (Manual)
1. Open Automator
2. File → New → Quick Action
3. Add "Run Shell Script" action
4. Script:
```bash
open -a "QRES Studio" "$@"
```
5. Save as "Compress with QRES"

**Result**: Right-click file → Services → "Compress with QRES"

### Installation (Automated)
```bash
cp macos-quick-action.plist ~/Library/Services/Compress\ with\ QRES.workflow/Contents/document.wflow
```

---

## Linux (Nautilus/Dolphin)

### Nautilus Script
**File**: `~/.local/share/nautilus/scripts/Compress with QRES`
```bash
#!/bin/bash
qres-studio "$@"
```
```bash
chmod +x ~/.local/share/nautilus/scripts/Compress\ with\ QRES
```

### Dolphin Service Menu
**File**: `~/.local/share/kservices5/qres-compress.desktop`
```ini
[Desktop Entry]
Type=Service
ServiceTypes=KonqPopupMenu/Plugin
MimeType=all/all;
Actions=compress

[Desktop Action compress]
Name=Compress with QRES
Icon=qres-studio
Exec=qres-studio %U
```
