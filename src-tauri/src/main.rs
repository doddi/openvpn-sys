// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate openvpn_sys;

use std::{env};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Mutex};
use bytes::Bytes;
use tauri::{CustomMenuItem, Icon, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent};
use tauri::utils::debug_eprintln;
use openvpn_sys::create_vpn_connector;
use openvpn_sys::prelude::*;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

static mut CONNECTOR: Option<Mutex<Box<dyn VpnConnector>>> = None;

#[derive(Debug)]
enum VpnCommands {
    Connect,
    Disconnect,
    Status,
}


struct ChannelConfiguration {
    tx_handler: Sender<VpnCommands>,
    rx_handler: Receiver<VpnCommands>,
}

#[derive(Debug, Clone)]
struct AppState {
    vpn_state: VpnStatus
}

async fn create_channel() -> Sender<VpnCommands> {
    let (tx_handler, rx_handler) = mpsc::channel(32);

    create_vpn_manager(rx_handler).await;
    debug_eprintln!("create_channel completed");

    tx_handler
}

async fn create_vpn_manager(mut receiver: Receiver<VpnCommands>) {
    debug_eprintln!("vpn_manger: Started");
    let mut vpn_connector = create_vpn_connector(ConnectorType::Open);
    debug_eprintln!("vpn_manager: connector created");

    let manager = tokio::spawn(async move {
        debug_eprintln!("Spawned manager");
        while let Some(command) = receiver.recv().await {
            use VpnCommands::*;

            debug_eprintln!("vpn_manager received event: {:?}", command);
            match command as VpnCommands {
                Connect => vpn_connector.connect(),
                Disconnect => vpn_connector.disconnect(),
                Status => vpn_connector.status()
            };
        }
    });

    debug_eprintln!("create_vpn_manager: created");
}

#[tauri::command]
async fn connect(app: tauri::AppHandle,
                 tx_handler: tauri::State<'_, Mutex<Sender<VpnCommands>>>,
                 status: tauri::State<'_, Mutex<AppState>>
) -> Result<(), ()>{
    debug_eprintln!("connect: clicked");
    let mut guarded_status = status.lock().unwrap().clone().vpn_state;

    debug_eprintln!("connect: clicked with status {:?}", guarded_status);
    let sender = tx_handler.lock().unwrap().clone();
    match guarded_status {
        VpnStatus::Disconnected => {
            debug_eprintln!("Connect clicked");
            // change_state(&app, guarded_status.deref_mut(), VpnStatus::Initialising);
            sender.send(VpnCommands::Connect).await.expect("Attempted to send connect message");
        }
        VpnStatus::Connected => {
            debug_eprintln!("Disconnect clicked");
            // change_state(&app, guarded_status.deref_mut(), VpnStatus::Disconnecting);
            sender.send(VpnCommands::Disconnect).await.expect("Attempted to send disconnect message");
        }
        _ => {}
    }
    Ok(())
}

fn change_state(app: &tauri::AppHandle,
                state: &mut AppState,
                new_connection_state: VpnStatus) {
    state.vpn_state = new_connection_state.clone();
    update_icon(&app, new_connection_state);
    emit_connection_status(&app, state);
}

fn emit_connection_status(app: &tauri::AppHandle, state: &AppState) {
    let unwrapped_state = state.deref().clone();
    debug_eprintln!("emit_connection_status: {:?}", unwrapped_state);

    // TODO Should not use expect
    app.emit_to("main", "connect_status", unwrapped_state.vpn_state.clone())
        .expect(format!("Unable to send {} message", unwrapped_state.vpn_state).as_str());
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
                      tx_handler: tauri::State<'_, Mutex<Sender<VpnCommands>>>,
                      status: tauri::State<'_, Mutex<AppState>>) -> Result<VpnStatus, ()> {
    debug_eprintln!("entered check_status");
    let mut guarded_status = status.lock().unwrap();

    // unsafe {
    //     match &CONNECTOR {
    //         Some(connector) => {
    //             let vpn_status = connector.lock().unwrap().status();
    //
    //             debug_eprintln!("check_status: {:?}", vpn_status);
    //             match vpn_status {
    //                 VpnStatus::Disconnected => {
    //                     change_state(&app, guarded_status.deref_mut(), VpnStatus::Disconnected);
    //                 }
    //                 VpnStatus::Connected => {
    //                     change_state(&app, guarded_status.deref_mut(), VpnStatus::Connected);
    //                 }
    //                 VpnStatus::Connecting | VpnStatus::Authenticating => {
    //                     change_state(&app, guarded_status.deref_mut(), VpnStatus::Connecting);
    //                 }
    //                 _ => {
    //                     change_state(&app, guarded_status.deref_mut(), VpnStatus::Error("".to_string()));
    //                 }
    //             }
    //             Ok(guarded_status.vpn_state.clone())
    //         }
    //         _ => {
    //             Err(())
    //         }
    //     }
    // }
    Err(())
}

#[tokio::main]
async fn main() {
    let open = CustomMenuItem::new("open".to_string(), "Open");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(open)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new()
        .with_menu(tray_menu);

    let channel_configuration = Mutex::new(create_channel().await);
    let managed_state = Mutex::new(AppState { vpn_state: VpnStatus::Disconnected });

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            match event {
                SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "open" => {
                            app.get_window("main").unwrap().show().unwrap();
                        }
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
        .manage(channel_configuration)
        .invoke_handler(tauri::generate_handler![connect, check_status])
        .on_window_event(|event| {
            match event.event() {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    event.window().hide().unwrap();
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
