# QRES Studio Setup - Configuration Files

## Run these commands to create all config files:

```powershell
cd C:\Dev\QRES\qres-studio

# Create directories
New-Item -ItemType Directory -Force -Path ui\src
New-Item -ItemType Directory -Force -Path icons

# 1. UI Package.json
@'
{
  "name": "qres-studio-ui",
  "version": "2.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "svelte": "^4.2.0",
    "vite": "^5.0.0"
  }
}
'@ | Out-File -FilePath ui\package.json -Encoding UTF8

# 2. Vite Config
@'
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
'@ | Out-File -FilePath ui\vite.config.js -Encoding UTF8

# 3. Svelte Config
@'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default {
  preprocess: vitePreprocess(),
};
'@ | Out-File -FilePath ui\svelte.config.js -Encoding UTF8

# 4. HTML Entry
@'
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>QRES Studio</title>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="/src/main.js"></script>
</body>
</html>
'@ | Out-File -FilePath ui\index.html -Encoding UTF8

# 5. Main JS Entry
@'
import App from './App.svelte';

const app = new App({
  target: document.getElementById('app'),
});

export default app;
'@ | Out-File -FilePath ui\src\main.js -Encoding UTF8

# 6. Tauri Config
@'
{
  "productName": "QRES Studio",
  "version": "2.0.0",
  "identifier": "com.qres.studio",
  "build": {
    "beforeDevCommand": "cd ui && npm run dev",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "cd ui && npm run build",
    "frontendDist": "../ui/dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/icon.png"],
    "category": "Utility",
    "shortDescription": "AI-Powered Compression",
    "longDescription": "QRES Studio - Real-time compression visualization"
  },
  "app": {
    "windows": [{
      "title": "QRES Studio",
      "width": 1000,
      "height": 700
    }]
  }
}
'@ | Out-File -FilePath tauri.conf.json -Encoding UTF8

# 7. Create a simple icon placeholder (base64 encoded 1x1 PNG)
# For production, replace with a real 512x512 PNG
$iconBytes = [Convert]::FromBase64String("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==")
[System.IO.File]::WriteAllBytes("$PWD\icons\icon.png", $iconBytes)

Write-Host "✅ All config files created!"
Write-Host ""
Write-Host "Next steps:"
Write-Host "1. cd ui"
Write-Host "2. npm install"
Write-Host "3. cd .."
Write-Host "4. cargo install tauri-cli"
Write-Host "5. cargo tauri build"
```

## Alternative: Manual Creation

If the script fails, create these files manually:

### ui/package.json
```json
{
  "name": "qres-studio-ui",
  "version": "2.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "svelte": "^4.2.0",
    "vite": "^5.0.0"
  }
}
```

### ui/vite.config.js
```javascript
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
```

### ui/svelte.config.js
```javascript
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
export default { preprocess: vitePreprocess() };
```

### ui/index.html
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>QRES Studio</title>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="/src/main.js"></script>
</body>
</html>
```

### ui/src/main.js
```javascript
import App from './App.svelte';
const app = new App({ target: document.getElementById('app') });
export default app;
```

### tauri.conf.json
```json
{
  "productName": "QRES Studio",
  "version": "2.0.0",
  "identifier": "com.qres.studio",
  "build": {
    "beforeDevCommand": "cd ui && npm run dev",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "cd ui && npm run build",
    "frontendDist": "../ui/dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/icon.png"],
    "category": "Utility"
  },
  "app": {
    "windows": [{"title": "QRES Studio", "width": 1000, "height": 700}]
  }
}
```

## Icon Setup

The app icon (`app-icon.png`) is already in the project root. Generate all platform icons:

```powershell
npm run tauri icon app-icon.png
```

This creates all required icons (.ico for Windows, .icns for Mac, PNGs for Linux).

## Build Commands

```powershell
cd C:\Dev\QRES\qres-studio

# Install dependencies
cd ui
npm install
cd ..

# Build the app
cargo install tauri-cli
cargo tauri build

# Output will be in:
# target\release\bundle\msi\
```
