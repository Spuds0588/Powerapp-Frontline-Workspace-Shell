// Prevents an additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;

use config::{AppConfig, LayoutConfig};
use tauri::{
    webview::WebviewBuilder, window::WindowBuilder, LogicalPosition, LogicalSize, Manager,
    WebviewUrl, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use url::Url;

// Embed the configuration directly into the compiled binary.
const COMPILED_CONFIG: &str = include_str!("../../config.json");

type Rect = (f64, f64, f64, f64);

fn load_embedded_config() -> AppConfig {
    serde_json::from_str(COMPILED_CONFIG)
        .expect("[CONFIG][FATAL] Failed to parse embedded config.json. Verify JSON syntax.")
}

fn is_domain_allowed(target: &Url, whitelist: &[String]) -> bool {
    if whitelist.is_empty() {
        return true;
    }
    let Some(host) = target.host_str() else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    whitelist.iter().any(|allowed| {
        let allowed = allowed.to_ascii_lowercase();
        host == allowed || host.ends_with(&format!(".{allowed}"))
    })
}

fn navigation_guard(whitelist: Vec<String>, pane: &'static str) -> impl Fn(&Url) -> bool {
    move |url| {
        let allowed = is_domain_allowed(url, &whitelist);
        if allowed {
            println!("[NAVIGATION][ALLOWED] {pane} pane navigating: {url}");
        } else {
            println!("[SECURITY][BLOCKED] Navigation rejected on {pane} pane: {url}");
        }
        allowed
    }
}

fn calculate_layout(cfg: &LayoutConfig, total_w: f64, total_h: f64) -> (Rect, Option<Rect>) {
    if cfg.mode != "dual" {
        return ((0.0, 0.0, total_w, total_h), None);
    }

    // Keep the main pane usable, then clamp the sidebar to the configured width.
    let max_sidebar = (total_w * 0.75)
        .max(100.0)
        .min((total_w - 200.0).max(100.0));
    let sidebar_w = cfg.sidebar_width.clamp(100.0, max_sidebar);
    let main_w = (total_w - sidebar_w).max(200.0);

    if cfg.sidebar_position == "left" {
        (
            (sidebar_w, 0.0, main_w, total_h),
            Some((0.0, 0.0, sidebar_w, total_h)),
        )
    } else {
        (
            (0.0, 0.0, main_w, total_h),
            Some((main_w, 0.0, sidebar_w, total_h)),
        )
    }
}

fn main() {
    println!("[INIT] Frontline Workspace Shell launched.");
    let config = load_embedded_config();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        println!("[SECURITY] Supervisor exit activated (Ctrl+Shift+Q).");
                        app.exit(0);
                    }
                })
                .build(),
        )
        .setup(move |app| {
            let mut cfg = config;
            if cfg.layout.mode == "dual" && cfg.panes.secondary.is_none() {
                println!(
                    "[CONFIG][WARN] Dual mode has no secondary pane; falling back to single mode."
                );
                cfg.layout.mode = "single".to_string();
            }
            println!(
                "[CONFIG][EMBEDDED] Loaded profile: '{}' | Mode='{}' | Kiosk={}",
                cfg.title, cfg.layout.mode, cfg.kiosk_mode
            );

            let exit_shortcut =
                Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyQ);
            if let Err(err) = app.global_shortcut().register(exit_shortcut) {
                println!("[SECURITY][WARN] Could not register Ctrl+Shift+Q supervisor exit: {err}");
            }

            let mut win_builder = WindowBuilder::new(app, "root_window")
                .title(cfg.title.clone())
                .inner_size(cfg.window_width, cfg.window_height)
                .min_inner_size(640.0, 400.0);

            if cfg.kiosk_mode {
                println!("[SECURITY] Kiosk lockdown active. Decorations stripped.");
                win_builder = win_builder
                    .fullscreen(true)
                    .decorations(false)
                    .resizable(false);
            } else if cfg.fullscreen {
                win_builder = win_builder.fullscreen(true);
            }

            let window = win_builder.build()?;
            let scale = window.scale_factor()?;
            let client = window.inner_size()?.to_logical::<f64>(scale);
            let (primary_rect, secondary_rect) =
                calculate_layout(&cfg.layout, client.width, client.height);

            let context_menu_script = if cfg.disable_context_menu {
                "window.addEventListener('contextmenu', event => event.preventDefault());"
            } else {
                ""
            };

            let primary_url: Url = cfg
                .panes
                .primary
                .url
                .parse()
                .expect("[CONFIG][FATAL] Invalid primary pane URL.");
            let primary = WebviewBuilder::new("primary_view", WebviewUrl::External(primary_url))
                .initialization_script(context_menu_script)
                .on_navigation(navigation_guard(cfg.allowed_domains.clone(), "primary"));
            window.add_child(
                primary,
                LogicalPosition::new(primary_rect.0, primary_rect.1),
                LogicalSize::new(primary_rect.2, primary_rect.3),
            )?;

            if let (Some(pane), Some(rect)) = (&cfg.panes.secondary, secondary_rect) {
                let secondary_url: Url = pane
                    .url
                    .parse()
                    .expect("[CONFIG][FATAL] Invalid secondary pane URL.");
                let secondary =
                    WebviewBuilder::new("secondary_view", WebviewUrl::External(secondary_url))
                        .initialization_script(context_menu_script)
                        .on_navigation(navigation_guard(cfg.allowed_domains.clone(), "secondary"));
                window.add_child(
                    secondary,
                    LogicalPosition::new(rect.0, rect.1),
                    LogicalSize::new(rect.2, rect.3),
                )?;
            }

            let window_for_resize = window.clone();
            let layout = cfg.layout.clone();
            window.on_window_event(move |event| {
                let WindowEvent::Resized(physical_size) = event else {
                    return;
                };
                let factor = window_for_resize.scale_factor().unwrap_or(1.0);
                let logical = physical_size.to_logical::<f64>(factor);
                let (primary, secondary) = calculate_layout(&layout, logical.width, logical.height);

                if let Some(view) = window_for_resize.get_webview("primary_view") {
                    let _ = view.set_position(LogicalPosition::new(primary.0, primary.1));
                    let _ = view.set_size(LogicalSize::new(primary.2, primary.3));
                }
                if let (Some(view), Some(rect)) =
                    (window_for_resize.get_webview("secondary_view"), secondary)
                {
                    let _ = view.set_position(LogicalPosition::new(rect.0, rect.1));
                    let _ = view.set_size(LogicalSize::new(rect.2, rect.3));
                }

                println!(
                    "[LAYOUT][SYNC] Viewports snapped to: {}x{}",
                    logical.width, logical.height
                );
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("[FATAL] Error running Frontline Workspace Shell");
}
