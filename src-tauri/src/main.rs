// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// TODO How to move to a lib
mod openvpn;
mod vpn;
mod dummyvpn;

use std::{env, fmt};
use std::fmt::Formatter;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{CustomMenuItem, Icon, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent};
use tauri::utils::debug_eprintln;
use crate::dummyvpn::DummyVpn;
use crate::openvpn::OpenVpn;
use crate::vpn::{VpnConnector, VpnStatus};

#[derive(Copy, Clone, Debug, serde::Serialize, PartialEq)]
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

#[derive(Clone, serde::Serialize)]
struct OpenVpnState {
    connection_status: ConnectionStatus,
    vpn: OpenVpn
}

impl OpenVpnState {
    fn new(connection_status: ConnectionStatus) -> OpenVpnState {
        OpenVpnState {
            connection_status,
            vpn: OpenVpn::new(String::from("/home/mdodgson/work/sonatype/config/sonatype.ovpn"))
        }
    }
}

impl Default for OpenVpnState {
    fn default() -> Self {
        OpenVpnState::new(ConnectionStatus::Disconnected)
    }
}

#[tauri::command]
async fn connect(app: tauri::AppHandle,
                 state: tauri::State<'_, Mutex<OpenVpnState>>) -> Result<(), ()>{
    debug_eprintln!("connect: clicked");
    let mut guarded_state = state.lock().unwrap();

    debug_eprintln!("connect: clicked with status {:?}", guarded_state.connection_status);
    match guarded_state.connection_status {
        ConnectionStatus::Disconnected => {
            debug_eprintln!("Connect clicked");
            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Initialising);
            let vpn_response = guarded_state.vpn.connect();

            match vpn_response {
                VpnStatus::Connected => {
                    change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Connected);
                }
                VpnStatus::Connecting | VpnStatus::Authenticating => {
                    change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Connecting);
                }
                _ => {
                    change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Error);
                }
            }
        }
        ConnectionStatus::Connected => {
            debug_eprintln!("Disconnect clicked");
            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Disconnecting);

            let vpn_response = guarded_state.vpn.disconnect();
            match vpn_response {
                VpnStatus::Disconnected => {
                    change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Disconnected);
                }
                _ => {
                    change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Error);
                }
            }

        }
        _ => {}
    }
    Ok(())
}

fn change_state(app: &tauri::AppHandle, state: &mut OpenVpnState, new_connection_state: ConnectionStatus) {
    state.connection_status = new_connection_state;
    update_icon(&app, new_connection_state);
    emit_connection_status(&app, state);
}

fn emit_connection_status(app: &tauri::AppHandle, state: &mut OpenVpnState) {
    let unwrapped_state = state.deref().clone();
    debug_eprintln!("emit_connection_status: {:?}", unwrapped_state.connection_status);

    // TODO Should not use expect
    app.emit_to("main", "connect_status", unwrapped_state.clone())
        .expect(format!("Unable to send {} message", unwrapped_state.connection_status).as_str());
}

fn update_icon(app: &tauri::AppHandle, new_connection_state: ConnectionStatus) {
    app.tray_handle().set_icon(get_icon(new_connection_state)).unwrap();
}

fn get_icon(connection_state: ConnectionStatus) -> Icon {
    debug_eprintln!("get_icon: {}", connection_state.to_string());
    match connection_state {
        ConnectionStatus::Disconnected => Icon::File(PathBuf::from("icons/SON_hexagon_disconnected.png")),
        ConnectionStatus::Connected => Icon::File(PathBuf::from("icons/SON_hexagon_connected.png")),
        ConnectionStatus::Error => Icon::File(PathBuf::from("icons/SON_hexagon_error.png")),
        _ => Icon::File(PathBuf::from("icons/SON_hexagon_intermediate.png"))
    }
}

#[tauri::command]
async fn check_status(app: tauri::AppHandle,
                      state: tauri::State<'_, Mutex<OpenVpnState>>) -> Result<ConnectionStatus, ()>{
    // return Ok(ConnectionStatus::Disconnected);
    debug_eprintln!("entered check_status");
    let mut guarded_state = state.lock().unwrap();

    let vpn_status = guarded_state.vpn.status();
    debug_eprintln!("check_status: {:?}", vpn_status);
    match vpn_status {
        VpnStatus::Disconnected => {
            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Disconnected);
        }
        VpnStatus::Connected => {
            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Connected);
        }
        VpnStatus::Connecting | VpnStatus::Authenticating => {
            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Connecting);
        }
        _ => {
            change_state(&app, guarded_state.deref_mut(), ConnectionStatus::Error);
        }
    }
    Ok(guarded_state.connection_status)
}

fn main() {
    // env::set_current_dir("/home/mdodgson/work/sonatype/config")
    //     .expect("Unable to set working directory to configuration location");

    let open = CustomMenuItem::new("open".to_string(), "Open");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(open)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new()
        .with_menu(tray_menu);

    let managed_state = Mutex::new(OpenVpnState::default());
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
        .manage(managed_state)
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
