// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod openvpn;
mod vpn;

use std::{env, fmt};
use std::fmt::Formatter;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Mutex};
use std::thread::sleep;
use std::time::Duration;
use tauri::{AppHandle, CustomMenuItem, Icon, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent};
use tauri::utils::debug_eprintln;


#[tauri::command]
async fn connect(app: AppHandle,
                 state: tauri::State<'_, Mutex<OpenVpnState>>) -> Result<(), ()>{
    let mut guarded_state = state.lock().unwrap();

    match guarded_state.connection_status {
        ConnectionStatus::Disconnected => {
            debug_eprintln!("Connect clicked");
            sleep(Duration::from_secs(1));

            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Initialising);

            sleep(Duration::from_secs(2));
            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Connecting);

            sleep(Duration::from_secs(5));
            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Connected);
        }
        ConnectionStatus::Connected => {
            debug_eprintln!("Disconnect clicked");
            sleep(Duration::from_secs(1));
            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Disconnecting);

            sleep(Duration::from_secs(1));
            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Disconnected);
        }
        _ => {}
    }
    Ok(())
}

fn change_state(app: &AppHandle, state: &mut OpenVpnState, newConnectionState: ConnectionStatus) {
    state.connection_status = newConnectionState;
    emit_connection_status(&app, state);
}

fn update_icon(app: &AppHandle, new_connection_state: ConnectionStatus) {
    app.tray_handle().set_icon(new_connection_state.get_icon()).unwrap();
}

trait IconChoice {
    fn get_icon(&self) -> Icon;
}

impl IconChoice for ConnectionStatus {
    fn get_icon(&self) -> Icon {
        return match self {
            ConnectionStatus::Disconnected => Icon::File(PathBuf::from("icons/SON_hexagon_disconnected.png")),
            ConnectionStatus::Connected => Icon::File(PathBuf::from("icons/SON_hexagon_connected.png")),
            ConnectionStatus::Error => Icon::File(PathBuf::from("icons/SON_hexagon_error.png")),
            _ => Icon::File(PathBuf::from("icons/SON_hexagon_intermediate.png"))
        }
    }
}


#[tauri::command]
async fn check_status(
    state: tauri::State<'_, Mutex<OpenVpnState>>) -> Result<ConnectionStatus, ()>{
    sleep(Duration::from_secs(2));
    Ok(state.lock().unwrap().connection_status)
}

fn emit_connection_status(app: &AppHandle, state: &mut OpenVpnState) {
    let unwrapped_state = state.deref().clone();
    debug_eprintln!("{}", unwrapped_state.connection_status);
    app.emit_to("main", "connect_status", unwrapped_state)
        .expect(format!("Unable to send {} message", unwrapped_state.connection_status).as_str());
}

#[derive(Copy, Clone, serde::Serialize, PartialEq)]
enum ConnectionStatus {
    Disconnected,
    Initialising,
    Connecting,
    Connected,
    Disconnecting,
    Error
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionStatus::Connected => { write!(f, "Connected") }
            ConnectionStatus::Initialising => { write!(f, "Initialising") }
            ConnectionStatus::Disconnected => { write!(f, "Disconnected") }
            ConnectionStatus::Connecting => { write!(f, "Connecting") }
            ConnectionStatus::Disconnecting => { write!(f, "Disconnecting") }
            ConnectionStatus::Error => { write!(f, "Error") }
        }
    }
}

#[derive(Copy, Clone, serde::Serialize)]
struct OpenVpnState {
    config_file: String,
    connection_status: ConnectionStatus
}

impl OpenVpnState {
    fn new(config_file: String,
           connection_status: ConnectionStatus) -> OpenVpnState {
        OpenVpnState { config_file, connection_status }
    }
}

impl Default for OpenVpnState {
    fn default() -> Self {
        OpenVpnState::new(
            String::from("sonatype.ovpn"),
            ConnectionStatus::Disconnected)
    }
}

fn main() {
    env::set_current_dir("/home/mdodgson/work/sonatype/config")
        .expect("Unable to set working directory to configuration location");

    let open = CustomMenuItem::new("open".to_string(), "Open");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(open)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new()
        .with_menu(tray_menu);

    tauri::Builder::default()
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
        .manage(Mutex::new(OpenVpnState::default()))
        .invoke_handler(tauri::generate_handler![connect, check_status])
        .on_window_event(|event| {
            match event.event() {
                WindowEvent::CloseRequested { api, ..} => {
                    api.prevent_close();
                    event.window().hide().unwrap();
                },
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
