// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate openvpn_sys;

use openvpn_sys::create_vpn_connector;
use openvpn_sys::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::utils::debug_eprintln;
use tauri::{
    generate_context, AppHandle, CustomMenuItem, Icon, Manager, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum VpnCommands {
    Connect,
    Disconnect,
    Status {
        responder: oneshot::Sender<VpnStatus>,
    },
}

async fn create_vpn_manager() -> Sender<VpnCommands> {
    debug_eprintln!("vpn_manger: Started");
    let (tx_handler, mut rx_handler) = mpsc::channel(32);

    let mut vpn_connector = create_vpn_connector(ConnectorType::Open);
    debug_eprintln!("vpn_manager: connector created");

    let _manager = tokio::spawn(async move {
        debug_eprintln!("Spawned manager");
        while let Some(command) = rx_handler.recv().await {
            use VpnCommands::*;

            debug_eprintln!("vpn_manager received event: {:?}", command);
            match command as VpnCommands {
                Connect => {
                    vpn_connector.connect();
                }
                Disconnect => {
                    vpn_connector.disconnect();
                }
                Status { responder } => {
                    let vpn_status = vpn_connector.status();
                    responder.send(vpn_status).unwrap();
                }
            };
        }
    });

    tx_handler
}

async fn create_status_monitor(app: AppHandle, tx_handler: Sender<VpnCommands>) {
    debug_eprintln!("status_monitor: entered");

    let _monitor = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let (tx_status_handler, rx_status_handler) = oneshot::channel();
            let status_command = VpnCommands::Status {
                responder: tx_status_handler,
            };

            debug_eprintln!("status_monitor: requesting status");
            tx_handler.send(status_command).await.unwrap();
            debug_eprintln!("status_monitor: waiting for status");
            let vpn_status = rx_status_handler.await.unwrap();
            debug_eprintln!("status_monitor: current status {}", vpn_status);

            change_state(&app, vpn_status);
        }
    });
}

#[tauri::command]
async fn connect(
    command: &str,
    tx_handler: tauri::State<'_, Mutex<Sender<VpnCommands>>>,
) -> Result<(), ()> {
    debug_eprintln!("connect: clicked");

    debug_eprintln!("connect: clicked with command {:?}", command);
    let sender = tx_handler.lock().unwrap().clone();
    match command {
        "connect" => {
            debug_eprintln!("Connect clicked");
            // change_state(&app, guarded_status.deref_mut(), VpnStatus::Initialising);
            sender
                .send(VpnCommands::Connect)
                .await
                .expect("Attempted to send connect message");
        }
        "disconnect" => {
            debug_eprintln!("Disconnect clicked");
            // change_state(&app, guarded_status.deref_mut(), VpnStatus::Disconnecting);
            sender
                .send(VpnCommands::Disconnect)
                .await
                .expect("Attempted to send disconnect message");
        }
        _ => {}
    }
    Ok(())
}

fn change_state(app: &AppHandle, state: VpnStatus) {
    update_icon(app, state.clone());
    emit_connection_status(app, state);
}

fn emit_connection_status(app: &AppHandle, state: VpnStatus) {
    debug_eprintln!("emit_connection_status: {:?}", state);

    // TODO Should not use expect
    app.emit_all( "connect_status", state.clone())
        .expect(format!("Unable to send {} message", state).as_str());
}

fn update_icon(app: &AppHandle, state: VpnStatus) {
    app.tray_handle().set_icon(get_icon(state)).unwrap();
}

fn get_icon(connection_state: VpnStatus) -> Icon {
    // debug_eprintln!("get_icon: {}", connection_state.to_string());
    match connection_state {
        VpnStatus::Disconnected => Icon::File(PathBuf::from("icons/SON_hexagon_disconnected.png")),
        VpnStatus::Connected => Icon::File(PathBuf::from("icons/SON_hexagon_connected.png")),
        VpnStatus::Error(_) => Icon::File(PathBuf::from("icons/SON_hexagon_error.png")),
        _ => Icon::File(PathBuf::from("icons/SON_hexagon_intermediate.png")),
    }
}

#[tokio::main]
async fn main() {
    let open = CustomMenuItem::new("open".to_string(), "Open");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(open)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    let vpn_tx_handler = create_vpn_manager().await;
    // TODO Does this now need to be in a Mutex?
    let channel_configuration = Mutex::new(vpn_tx_handler.clone());

    let app = tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
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
        })
        .manage(channel_configuration)
        .invoke_handler(tauri::generate_handler![connect])
        .on_window_event(|event| {
            if let WindowEvent::CloseRequested { api, .. } = event.event() {
                api.prevent_close();
                event.window().hide().unwrap();
            }
        })
        .build(generate_context!())
        .unwrap();

    create_status_monitor(app.handle(), vpn_tx_handler.clone()).await;

    app.run(|_app_handle, _event| {});
}
