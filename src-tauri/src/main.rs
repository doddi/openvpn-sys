// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fmt;
use std::fmt::Formatter;
use std::thread::sleep;
use std::time::Duration;
use tauri::{AppHandle, CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tauri::utils::debug_eprintln;

#[tauri::command]
async fn connect(app: AppHandle) {
    debug_eprintln!("Connect clicked");
    sleep(Duration::from_secs(1));
    emit_connection_status(&app, ConnectionStatus::Initialising);

    sleep(Duration::from_secs(2));
    emit_connection_status(&app, ConnectionStatus::Connecting);

    sleep(Duration::from_secs(5));
    emit_connection_status(&app, ConnectionStatus::Connected);
}

fn emit_connection_status(app: &AppHandle, status: ConnectionStatus) {
    debug_eprintln!("{}", status);
    app.emit_to("main", "connect_status", status)
        .expect(format!("Unable to send {} message", status).as_str());
}

#[derive(Copy, Clone, serde::Serialize)]
enum ConnectionStatus {
    Disconnected,
    Initialising,
    Connecting,
    Connected,
    Disconnecting
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionStatus::Connected => { write!(f, "Connected") }
            ConnectionStatus::Initialising => { write!(f, "Initialising") }
            ConnectionStatus::Disconnected => { write!(f, "Disconnected") }
            ConnectionStatus::Connecting => { write!(f, "Connecting") }
            ConnectionStatus::Disconnecting => { write!(f, "Disconnecting") }
        }
    }
}

fn main() {
    let open = CustomMenuItem::new("open".to_string(), "Open");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(open)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new()
        .with_menu(tray_menu);

    let app = tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            match event {
                SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "open" => {
                            app.get_window("main").unwrap().show().unwrap();
                        },
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![connect])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(move |_app_handle, run_event| {
        if let RunEvent::ExitRequested { api, .. } = &run_event {
            api.prevent_exit();
        }
    });
}
