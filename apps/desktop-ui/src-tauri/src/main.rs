#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::TcpListener;
use std::sync::OnceLock;
use tauri::Manager;

static DAEMON_BOOTSTRAPPED: OnceLock<()> = OnceLock::new();
static DAEMON_BASE_URL: OnceLock<String> = OnceLock::new();

const DEFAULT_DAEMON_HOST: &str = "127.0.0.1";
const DEFAULT_DAEMON_PORT: u16 = 11435;
const DAEMON_PORT_SCAN_LIMIT: u16 = 12;

fn should_bootstrap_embedded_daemon() -> bool {
    match std::env::var("MLX_PILOT_DISABLE_EMBEDDED_DAEMON") {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            !(normalized == "1"
                || normalized == "true"
                || normalized == "yes"
                || normalized == "on")
        }
        Err(_) => true,
    }
}

fn resolve_embedded_daemon_bind_addr() -> String {
    if let Ok(bind_addr) = std::env::var("APP_BIND_ADDR") {
        let trimmed = bind_addr.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    for port in DEFAULT_DAEMON_PORT..=DEFAULT_DAEMON_PORT + DAEMON_PORT_SCAN_LIMIT {
        if let Ok(listener) = TcpListener::bind((DEFAULT_DAEMON_HOST, port)) {
            drop(listener);
            return format!("{DEFAULT_DAEMON_HOST}:{port}");
        }
    }

    if let Ok(listener) = TcpListener::bind((DEFAULT_DAEMON_HOST, 0)) {
        if let Ok(addr) = listener.local_addr() {
            let host = match addr.ip() {
                std::net::IpAddr::V4(ip) => ip.to_string(),
                std::net::IpAddr::V6(ip) => format!("[{ip}]"),
            };
            drop(listener);
            return format!("{host}:{}", addr.port());
        }
    }

    format!("{DEFAULT_DAEMON_HOST}:{DEFAULT_DAEMON_PORT}")
}

fn store_daemon_url(bind_addr: &str) -> String {
    let daemon_url = format!("http://{bind_addr}");
    let _ = DAEMON_BASE_URL.set(daemon_url.clone());
    std::env::set_var("APP_BIND_ADDR", bind_addr);
    std::env::set_var("MLX_PILOT_DAEMON_URL", &daemon_url);
    daemon_url
}

fn current_daemon_url() -> String {
    if let Some(url) = DAEMON_BASE_URL.get() {
        return url.clone();
    }

    if let Ok(url) = std::env::var("MLX_PILOT_DAEMON_URL") {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            let normalized = trimmed.to_string();
            let _ = DAEMON_BASE_URL.set(normalized.clone());
            return normalized;
        }
    }

    if let Ok(bind_addr) = std::env::var("APP_BIND_ADDR") {
        let trimmed = bind_addr.trim();
        if !trimmed.is_empty() {
            return store_daemon_url(trimmed);
        }
    }

    store_daemon_url(&format!("{DEFAULT_DAEMON_HOST}:{DEFAULT_DAEMON_PORT}"))
}

fn daemon_bootstrap_script() -> String {
    let daemon_url_json =
        serde_json::to_string(&current_daemon_url()).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        r#"
        window.__MLX_PILOT_DAEMON_URL__ = {daemon_url_json};
        try {{
          localStorage.setItem("mlxPilotDaemonUrl", window.__MLX_PILOT_DAEMON_URL__);
        }} catch (_error) {{}}
        window.dispatchEvent(new CustomEvent("mlx-pilot-daemon-ready", {{
          detail: {{ url: window.__MLX_PILOT_DAEMON_URL__ }}
        }}));
        "#
    )
}

fn inject_daemon_bootstrap(window: &tauri::WebviewWindow) {
    let script = daemon_bootstrap_script();
    let _ = window.eval(&script);
}

fn inject_daemon_bootstrap_into_webview(webview: &tauri::Webview) {
    let script = daemon_bootstrap_script();
    let _ = webview.eval(&script);
}

fn bootstrap_embedded_daemon() {
    if !should_bootstrap_embedded_daemon() {
        let _ = current_daemon_url();
        return;
    }

    let bind_addr = resolve_embedded_daemon_bind_addr();
    store_daemon_url(&bind_addr);

    if DAEMON_BOOTSTRAPPED.set(()).is_err() {
        return;
    }

    tauri::async_runtime::spawn(async {
        if let Err(error) = mlx_ollama_daemon::run().await {
            eprintln!("embedded daemon failed to start: {error}");
        }
    });
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            bootstrap_embedded_daemon();

            // Get the main webview window
            if let Some(webview_window) = app.get_webview_window("main") {
                inject_daemon_bootstrap(&webview_window);

                // Enable devtools only in debug builds
                #[cfg(debug_assertions)]
                webview_window.open_devtools();

                // Inject JS to block browser-like behaviors in release mode
                #[cfg(not(debug_assertions))]
                {
                    let _ = webview_window.eval(
                        r#"
                        // Block context menu (right-click)
                        document.addEventListener('contextmenu', function(e) {
                            e.preventDefault();
                        }, true);

                        // Block browser keyboard shortcuts
                        document.addEventListener('keydown', function(e) {
                            // Ctrl+Shift+I (DevTools)
                            if (e.ctrlKey && e.shiftKey && e.key === 'I') { e.preventDefault(); }
                            // Ctrl+Shift+J (Console)
                            if (e.ctrlKey && e.shiftKey && e.key === 'J') { e.preventDefault(); }
                            // Ctrl+U (View Source)
                            if (e.ctrlKey && e.key === 'u') { e.preventDefault(); }
                            // F12
                            if (e.key === 'F12') { e.preventDefault(); }
                        }, true);
                        "#,
                    );
                }
            }
            Ok(())
        })
        .on_page_load(|window, _payload| {
            inject_daemon_bootstrap_into_webview(window);
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
