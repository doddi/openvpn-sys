// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate openvpn_sys;

use std::{env};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Mutex};
use tauri::{CustomMenuItem, Icon, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent};
use tauri::utils::debug_eprintln;
use openvpn_sys::create_vpn_connector;
use openvpn_sys::prelude::*;

static mut CONNECTOR: Option<Mutex<Box<dyn VpnConnector>>> = None;

#[tauri::command]
async fn connect(app: tauri::AppHandle,
                 status: tauri::State<'_, Mutex<VpnStatus>>) -> Result<(), ()>{
    debug_eprintln!("connect: clicked");
    let mut guarded_status = status.lock().unwrap();

    debug_eprintln!("connect: clicked with status {:?}", guarded_status);
    match guarded_status.deref() {
        VpnStatus::Disconnected => {
            debug_eprintln!("Connect clicked");
            change_state(&app, guarded_status.deref_mut(), VpnStatus::Initialising);
            unsafe {
                match &CONNECTOR {
                    Some(connector) => {
                        let vpn_response = connector.lock().unwrap().connect();
                        match vpn_response {
                            VpnStatus::Connected => {
                                change_state(&app, guarded_status.deref_mut(), VpnStatus::Connected);
                            }
                            VpnStatus::Connecting | VpnStatus::Authenticating => {
                                change_state(&app, guarded_status.deref_mut(), VpnStatus::Connecting);
                            }
                            _ => {
                                change_state(&app, guarded_status.deref_mut(), VpnStatus::Error("".to_string()));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        VpnStatus::Connected => {
            debug_eprintln!("Disconnect clicked");
            change_state(&app, guarded_status.deref_mut(), VpnStatus::Disconnecting);

            unsafe {
                match &CONNECTOR {
                    Some(connector) => {
                        let vpn_response = connector.lock().unwrap().disconnect();
                        match vpn_response {
                            VpnStatus::Disconnected => {
                                change_state(&app, guarded_status.deref_mut(), VpnStatus::Disconnected);
                            }
                            _ => {
                                change_state(&app, guarded_status.deref_mut(), VpnStatus::Error("Error disconnecting".to_string()));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn change_state(app: &tauri::AppHandle,
                state: &mut VpnStatus,
                new_connection_state: VpnStatus) {
    *state = new_connection_state.clone();
    update_icon(&app, new_connection_state);
    emit_connection_status(&app, state);
}

fn emit_connection_status(app: &tauri::AppHandle, state: &mut VpnStatus) {
    let unwrapped_state = state.deref().clone();
    debug_eprintln!("emit_connection_status: {:?}", unwrapped_state);

    // TODO Should not use expect
    app.emit_to("main", "connect_status", unwrapped_state.clone())
        .expect(format!("Unable to send {} message", unwrapped_state).as_str());
}

fn update_icon(app: &tauri::AppHandle, new_connection_state: VpnStatus) {
    app.tray_handle().set_icon(get_icon(new_connection_state)).unwrap();
}

fn get_icon(connection_state: VpnStatus) -> Icon {
    debug_eprintln!("get_icon: {}", connection_state.to_string());
    match connection_state {
        VpnStatus::Disconnected => Icon::File(PathBuf::from("icons/SON_hexagon_disconnected.png")),
        VpnStatus::Connected => Icon::File(PathBuf::from("icons/SON_hexagon_connected.png")),
        VpnStatus::Error(_) => Icon::File(PathBuf::from("icons/SON_hexagon_error.png")),
        _ => Icon::File(PathBuf::from("icons/SON_hexagon_intermediate.png"))
    }
}

#[tauri::command]
async fn check_status(app: tauri::AppHandle,
                      status: tauri::State<'_, Mutex<VpnStatus>>) -> Result<VpnStatus, ()>{
    // return Ok(ConnectionStatus::Disconnected);
    debug_eprintln!("entered check_status");
    let mut guarded_status = status.lock().unwrap();

    unsafe {
        match &CONNECTOR {
            Some(connector) => {
                let vpn_status = connector.lock().unwrap().status();

                debug_eprintln!("check_status: {:?}", vpn_status);
                match vpn_status {
                    VpnStatus::Disconnected => {
                        change_state(&app, guarded_status.deref_mut(), VpnStatus::Disconnected);
                    }
                    VpnStatus::Connected => {
                        change_state(&app, guarded_status.deref_mut(), VpnStatus::Connected);
                    }
                    VpnStatus::Connecting | VpnStatus::Authenticating => {
                        change_state(&app, guarded_status.deref_mut(), VpnStatus::Connecting);
                    }
                    _ => {
                        change_state(&app, guarded_status.deref_mut(), VpnStatus::Error("".to_string()));
                    }
                }
                Ok(guarded_status.clone())
            }
            _ => {
                Err(())
            }
        }
    }
}

fn main() {
    unsafe {
        CONNECTOR = Option::from(Mutex::new(create_vpn_connector(ConnectorType::Open)));
    }

    let open = CustomMenuItem::new("open".to_string(), "Open");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(open)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new()
        .with_menu(tray_menu);

    let managed_state = Mutex::new(VpnStatus::Disconnected);

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
