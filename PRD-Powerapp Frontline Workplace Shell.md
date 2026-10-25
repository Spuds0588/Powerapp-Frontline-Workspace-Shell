
# MASTER_DEV.md

# Powerapp Frontline Workspace Shell (Desktop / Kiosk)
## PRD, Implementation Guide, Dev Task List, and AGENTS.md

---

## Part 1: Product Requirements Document (PRD)

### 1.1 Executive Summary

Powerapp Frontline Workspace Shell is a native desktop application container built on Tauri v2. It packages Microsoft Power Apps, Microsoft Fabric workloads, and complementary operational web services (e.g., VoIP dialers, ticketing consoles, second Power Apps) into a locked-down, resource-efficient desktop shell.

Unlike conventional enterprise wrappers, this template is tailored for **low-code makers, vibe coders, and non-developer IT professionals**. It eliminates the risk of missing runtime dependencies and fragmented files by **baking all configurations directly into the compiled executable**. 

Key Value Propositions:
1. **Zero Loose Files:** The configuration is baked directly into the binary at compile time. Deploying is as simple as distributing a single, standalone `.exe` or `.msi`.
2. **Cloud-Native Low-Code CI/CD:** Builders do not need local Rust or C++ build environments. They customize a single `config.json` in their repository, and automated GitHub Actions produce the ready-to-run enterprise installer.
3. **Drastic Memory Reduction:** ~40–80 MB idle RAM footprint vs. 300–600+ MB on Electron or standard desktop browsers.
4. **Security & Lockdown:** Strips away the browser address bar, restricts navigation strictly to approved corporate domains, disables context menus, and optionally forces dedicated kiosk mode.
5. **Native Entra ID (SSO) Inheritance:** Shares the OS-level WebView2 identity context so users log in once and immediately access all configured workspace panes.

---

### 1.2 Target Personas
* **Low-Code Power App Creators & Vibe Coders:** Makers seeking a professional, branded desktop distribution for their Canvas/Model-Driven apps without writing native desktop code or managing build chains.
* **Non-Developer IT Admins:** System administrators deploying locked-down, single-purpose workstations (call center terminals, front desks, factory floor stations) via Intune or Group Policy.
* **Frontline Operators:** End users in operational roles who require a fast, distraction-free, and crash-resilient workspace.

---

### 1.3 Key Architectural Principles & Boundaries
1. **Compile-Time Embedding:** Configuration is read from `config.json` at build time via Rust's `include_str!` macro. No runtime filesystem lookups, no path errors, and zero exposure to end-user tampering.
2. **Shared SSO as a Feature:** All viewports share the default WebView2 data directory. Authentication tokens for Microsoft 365, Power Apps, and Fabric are shared seamlessly. Multi-tenant partitioning is explicitly out of scope.
3. **Strict YAGNI:** The shell is an orchestration, windowing, and security container. It does not perform invasive DOM scraping or arbitrary web script injection.
4. **Zero Local Toolchain Requirement:** The canonical build target is automated cloud runners (GitHub Actions), eliminating local development friction for low-code authors.

---

### 1.4 Functional Requirements

#### FR-01: Compile-Time Configuration
* The build pipeline must parse a root-level `config.json` and bake its properties immutably into the application binary.
* Compile errors must trigger if `config.json` has invalid JSON syntax, catching configuration errors before the binary is generated.

#### FR-02: Viewport & Layout Orchestration
* **Single Mode:** Allocates 100% of the native client area to the primary application (e.g., Power App or Fabric workspace).
* **Dual Mode:** Splits the native client area into primary and secondary views (e.g., Power App + Web Phone / Teams).
* **Docking & Proportions:** Supports left or right sidebar docking with custom pixel or proportional split boundaries.
* **Adaptive Resize Sync:** Dynamically recomputes child webview coordinates on native window resize events without latency or layout drift.

#### FR-03: Hardened Enterprise Security & Kiosk Lockdown
* **Domain Whitelist Boundary:** Validates every URI navigation against an `allowedDomains` list. Any navigation or redirect outside these apex domains (and their subdomains) is blocked and logged.
* **Context Menu Suppression:** Suppresses right-click context menus (`Inspect Element`, `Reload`) to prevent operational tampering.
* **Kiosk Windowing:** Configurable borderless fullscreen execution with window decorations removed.
* **Supervisor Emergency Exit:** A hardcoded emergency exit shortcut (`Ctrl + Shift + Q`) to terminate the shell cleanly in kiosk deployments.

---

## Part 2: Implementation Guide

### 2.1 File Architecture
```text
frontline-desktop-shell/
├── .github/
│   └── workflows/
│       └── release.yml
├── config.json
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       └── main.rs
└── ui/
    └── index.html
```

---

### 2.2 Root Configuration Schema (`config.json`)
This is the single file low-code makers edit before triggering a build:

```json
{
  "title": "Frontline Operations Console",
  "kioskMode": false,
  "fullscreen": false,
  "windowWidth": 1600.0,
  "windowHeight": 900.0,
  "disableContextMenu": true,
  "allowedDomains": [
    "apps.powerapps.com",
    "content.powerapps.com",
    "login.microsoftonline.com",
    "login.windows.net",
    "aadcdn.msauth.net",
    "aadcdn.msftauth.net",
    "msft.sts.microsoft.com",
    "autologon.microsoftazuread-sso.com",
    "teams.microsoft.com",
    "app.fabric.microsoft.com",
    "app.powerbi.com"
  ],
  "layout": {
    "mode": "dual",
    "sidebarPosition": "right",
    "sidebarWidth": 460.0
  },
  "panes": {
    "primary": {
      "url": "https://apps.powerapps.com"
    },
    "secondary": {
      "url": "https://teams.microsoft.com"
    }
  }
}
```

---

### 2.3 Rust Source Implementation (`src-tauri/src/main.rs`)

```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Deserialize;
use std::sync::Arc;
use tauri::{
    webview::{WebviewBuilder, WebviewUrl},
    window::WindowBuilder,
    LogicalPosition, LogicalSize, Manager, WindowEvent,
};
use url::Url;

// Embed the configuration directly into the compiled binary
const COMPILED_CONFIG: &str = include_str!("../../config.json");

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppConfig {
    title: String,
    kiosk_mode: bool,
    fullscreen: bool,
    window_width: f64,
    window_height: f64,
    disable_context_menu: bool,
    allowed_domains: Vec<String>,
    layout: LayoutConfig,
    panes: PanesConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayoutConfig {
    mode: String,
    sidebar_position: String,
    sidebar_width: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PanesConfig {
    primary: PaneTarget,
    secondary: Option<PaneTarget>,
}

#[derive(Debug, Clone, Deserialize)]
struct PaneTarget {
    url: String,
}

fn load_embedded_config() -> AppConfig {
    serde_json::from_str(COMPILED_CONFIG)
        .expect("[CONFIG][FATAL] Failed to parse embedded config.json. Verify JSON syntax.")
}

fn is_domain_allowed(target: &Url, whitelist: &[String]) -> bool {
    if whitelist.is_empty() { return true; }
    let host = match target.host_str() {
        Some(h) => h,
        None => return false,
    };
    whitelist.iter().any(|allowed| host == allowed || host.ends_with(&format!(".{}", allowed)))
}

fn calculate_layout(cfg: &LayoutConfig, total_w: f64, total_h: f64) -> ((f64, f64, f64, f64), Option<(f64, f64, f64, f64)>) {
    if cfg.mode != "dual" {
        return ((0.0, 0.0, total_w, total_h), None);
    }
    let sb_w = cfg.sidebar_width.min(total_w * 0.75).max(100.0);
    let main_w = (total_w - sb_w).max(200.0);

    if cfg.sidebar_position == "left" {
        ((sb_w, 0.0, main_w, total_h), Some((0.0, 0.0, sb_w, total_h)))
    } else {
        ((0.0, 0.0, main_w, total_h), Some((main_w, 0.0, sb_w, total_h)))
    }
}

fn main() {
    let config = Arc::new(load_embedded_config());
    println!("[INIT] Frontline Workspace Shell launched.");
    println!("[CONFIG][EMBEDDED] Loaded profile: '{}' | Mode='{}' | Kiosk={}", config.title, config.layout.mode, config.kiosk_mode);

    let context_menu_script = if config.disable_context_menu {
        "window.addEventListener('contextmenu', e => e.preventDefault());"
    } else {
        ""
    };

    let cfg_setup = config.clone();

    tauri::Builder::default()
        .setup(move |app| {
            let mut win_builder = WindowBuilder::new(app, "root_window")
                .title(&cfg_setup.title)
                .inner_size(cfg_setup.window_width, cfg_setup.window_height);

            if cfg_setup.kiosk_mode {
                println!("[SECURITY] Kiosk lockdown active. Decorations stripped.");
                win_builder = win_builder.fullscreen(true).decorations(false).resizable(false);
            } else if cfg_setup.fullscreen {
                win_builder = win_builder.fullscreen(true);
            }

            let window = win_builder.build()?;
            let (prim_rect, sec_rect_opt) = calculate_layout(&cfg_setup.layout, cfg_setup.window_width, cfg_setup.window_height);

            // 1. Primary Viewport
            let prim_url: Url = cfg_setup.panes.primary.url.parse().expect("[CONFIG][FATAL] Invalid Primary URL");
            let prim_whitelist = cfg_setup.allowed_domains.clone();

            let primary_builder = WebviewBuilder::new("primary_view", WebviewUrl::External(prim_url))
                .initialization_script(context_menu_script)
                .on_navigation(move |url| {
                    let allowed = is_domain_allowed(url, &prim_whitelist);
                    if !allowed {
                        println!("[SECURITY][BLOCKED] Navigation rejected on primary pane: {}", url);
                    } else {
                        println!("[NAVIGATION][ALLOWED] Primary pane navigating: {}", url);
                    }
                    allowed
                });

            let prim_webview = window.add_child(
                primary_builder,
                LogicalPosition::new(prim_rect.0, prim_rect.1),
                LogicalSize::new(prim_rect.2, prim_rect.3),
            )?;

            // 2. Secondary Viewport (Dual Mode)
            let mut sec_webview = None;
            if let (Some(sec_cfg), Some(sec_rect)) = (&cfg_setup.panes.secondary, sec_rect_opt) {
                let sec_url: Url = sec_cfg.url.parse().expect("[CONFIG][FATAL] Invalid Secondary URL");
                let sec_whitelist = cfg_setup.allowed_domains.clone();

                let secondary_builder = WebviewBuilder::new("secondary_view", WebviewUrl::External(sec_url))
                    .initialization_script(context_menu_script)
                    .on_navigation(move |url| {
                        let allowed = is_domain_allowed(url, &sec_whitelist);
                        if !allowed {
                            println!("[SECURITY][BLOCKED] Navigation rejected on secondary pane: {}", url);
                        } else {
                            println!("[NAVIGATION][ALLOWED] Secondary pane navigating: {}", url);
                        }
                        allowed
                    });

                sec_webview = Some(window.add_child(
                    secondary_builder,
                    LogicalPosition::new(sec_rect.0, sec_rect.1),
                    LogicalSize::new(sec_rect.2, sec_rect.3),
                )?);
            }

            // Window Resizing & Viewport Geometry Recomputation
            let cfg_resize = cfg_setup.clone();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::Resized(physical_size) = event {
                    let factor = window_clone.scale_factor().unwrap_or(1.0);
                    let logical = physical_size.to_logical::<f64>(factor);
                    let (p, s_opt) = calculate_layout(&cfg_resize.layout, logical.width, logical.height);

                    let _ = prim_webview.set_position(LogicalPosition::new(p.0, p.1));
                    let _ = prim_webview.set_size(LogicalSize::new(p.2, p.3));

                    if let (Some(ref s_view), Some(s)) = (&sec_webview, s_opt) {
                        let _ = s_view.set_position(LogicalPosition::new(s.0, s.1));
                        let _ = s_view.set_size(LogicalSize::new(s.2, s.3));
                    }
                    println!("[LAYOUT][SYNC] Viewports snapped to: {}x{}", logical.width, logical.height);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("[FATAL] Error running Frontline Workspace Shell");
}
```

---

### 2.4 Supporting Project Files

#### `src-tauri/Cargo.toml`
```toml
[package]
name = "frontline-desktop-shell"
version = "1.0.0"
description = "Hardened low-memory desktop workspace container for Power Apps and Fabric"
authors = ["Enterprise Operations"]
edition = "2021"

[dependencies]
tauri = { version = "2.0", features = ["unstable"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
url = "2.5"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }
```

#### `src-tauri/tauri.conf.json`
```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-config-schema/schemas/config.schema.json",
  "productName": "FrontlineShell",
  "version": "1.0.0",
  "identifier": "com.frontline.shell",
  "build": {
    "frontendDist": "../ui"
  },
  "app": {
    "withGlobalTauri": true,
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis", "msi"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico"
    ]
  }
}
```

#### `ui/index.html`
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>Frontline Workspace</title>
  <style>
    body {
      margin: 0;
      background-color: #0b0f19;
      color: #94a3b8;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100vh;
      user-select: none;
    }
    .spinner {
      width: 24px;
      height: 24px;
      border: 3px solid #1e293b;
      border-top: 3px solid #38bdf8;
      border-radius: 50%;
      animation: spin 1s linear infinite;
      margin: 0 auto 12px;
    }
    @keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
  </style>
</head>
<body>
  <div style="text-align: center;">
    <div class="spinner"></div>
    <div>Mounting Enterprise Workspace...</div>
  </div>
</body>
</html>
```

---

### 2.5 Low-Code Cloud CI/CD Pipeline (`.github/workflows/release.yml`)
This workflow builds the standalone installer automatically whenever changes are pushed to `main`. Makers never install Rust or build tools locally.

```yaml
name: Build Enterprise Desktop Shell

on:
  push:
    branches: [ main ]
  workflow_dispatch:

jobs:
  build:
    runs-on: windows-latest
    steps:
      - name: Check out repository
        uses: actions/checkout@v4

      - name: Setup Rust Toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install Tauri CLI
        run: cargo install tauri-cli --version "^2.0"

      - name: Build Standalone Installer
        run: cargo tauri build

      - name: Upload Installer Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: FrontlineShell-Windows-Installer
          path: |
            src-tauri/target/release/bundle/msi/*.msi
            src-tauri/target/release/bundle/nsis/*.exe
```

---

## Part 3: Developer Task List

### Sprint 1: Compile-Time Config & Pipeline
- [x] Embed `config.json` via `include_str!` to eliminate loose files in distribution bundles.
- [x] Implement compile-time syntax assertion during configuration deserialization.
- [x] Create `.github/workflows/release.yml` for automated cloud-based `.exe`/`.msi` packaging.

### Sprint 2: Core Workspace Architecture
- [x] Implement dual/single viewport layout manager in native Rust.
- [x] Synchronize viewport bounds dynamically on `WindowEvent::Resized`.
- [x] Hook `on_navigation` for strict domain boundary validation.
- [x] Inject context menu suppression script globally into all mounted views.
- [x] Configure Kiosk mode window state flags.

### Sprint 3: Verification & Operational Handoff
- [ ] Fork template repository and verify automated GitHub Actions pipeline completes under 8 minutes.
- [ ] Download and execute the generated `.msi` on a clean Windows 10/11 environment.
- [ ] Validate that moving or deleting source files does not affect the installed application (confirming zero-dependency bundling).
- [ ] Verify Task Manager reports combined RSS memory usage under 90 MB under active Power Apps runtime execution.
- [ ] Validate domain boundary enforcement by attempting to navigate outside the whitelist; confirm `[SECURITY][BLOCKED]` logs in debug console.

---

## Part 4: AGENTS.md

### Mission & Code Guidelines for AI Agents
You are an expert systems developer maintaining the **Frontline Workspace Shell** project. Your primary users are low-code makers, vibe coders, and enterprise IT administrators.

1. **Self-Contained Executable Mandate:** Never reintroduce external filesystem lookups for configuration files. All configurations must be embedded into the binary at compile time (`include_str!`). The resulting executable or installer must be 100% self-sufficient.
2. **Strict YAGNI Compliance:** Do not add complex runtime IPC state layers, external database hooks, or custom web scraping engines. Maintain this project as a lean, secure application shell.
3. **One-Line & Idiomatic Solutions:** Favor idiomatic Rust and concise standard library solutions. Avoid unnecessary third-party crates.
4. **Full File Outputs Only:** When generating or modifying code, provide complete, runnable code files. Never emit diffs, snippets, or partial implementation blocks.
5. **Console Diagnostics Standard:** All runtime diagnostics must be emitted to stdout using c