use crate::vpn::{VpnConnector, VpnStatus};
use std::io::BufRead;
use std::process::{Command, Output};

enum OpenVpnStatsResult {
    Ok(String),
    Err(String),
}

#[derive(Clone, serde::Serialize)]
pub struct OpenVpn {
    config: String,
}

// TODO Wrap Command to allow testing
impl OpenVpn {
    pub fn new(config: String) -> Self {
        OpenVpn { config }
    }

    fn connect_open_vpn(&mut self) -> VpnStatus {
        let result = Command::new("openvpn3")
            .arg("session-start")
            .arg("--config")
            .arg(self.config.clone())
            .output();

        match result {
            Ok(output) => Self::check_connect_response(output),
            Err(_) => VpnStatus::Error(String::from("Unable to connect")),
        }
    }

    fn check_connect_response(output: Output) -> VpnStatus {
        if !output.stdout.is_empty() {
            let response = String::from_utf8(output.stdout).unwrap();
            if response.contains("** ERROR **") {
                return VpnStatus::Disconnected;
            }
            return VpnStatus::Connecting;
        }
        VpnStatus::Error(String::from_utf8(output.stderr).unwrap())
    }

    fn get_open_vpn_stats(&self) -> OpenVpnStatsResult {
        let result = Command::new("openvpn3")
            .arg("session-stats")
            .arg("--config")
            .arg(self.config.clone())
            .output();

        match result {
            Ok(output) => {
                let response = String::from_utf8(output.stdout).unwrap();
                if response.contains("** ERROR **") {
                    OpenVpnStatsResult::Err(String::from("Error getting stats"))
                } else {
                    OpenVpnStatsResult::Ok(response)
                }
            }
            Err(_) => OpenVpnStatsResult::Err(String::from("Unable to determine status")),
        }
    }

    fn determine_status(output: Output) -> VpnStatus {
        // debug_eprintln!("determine_status: {}", String::from_utf8(output.stdout.clone()).unwrap());
        for line in output.stdout.lines().flatten() {
            if line.trim_start().starts_with("Status:") {
                let status = line.trim().strip_prefix("Status:").unwrap().trim();
                return match status {
                    "Web authentication required to connect" => VpnStatus::Authenticating,
                    "Connection, Client connected" => VpnStatus::Connected,
                    _ => VpnStatus::Error(format!("Unknown session status, {}", status)),
                };
            } else if line.trim().contains("No sessions available") {
                return VpnStatus::Disconnected;
            }
        }
        VpnStatus::Error(String::from("Status line not found"))
    }

    fn get_open_vpn_sessions_list(&mut self) -> VpnStatus {
        let result = Command::new("openvpn3").arg("sessions-list").output();

        match result {
            Ok(output) => Self::determine_status(output),
            Err(_) => VpnStatus::Error(String::from("Unable to get session status")),
        }
    }

    fn disconnect_open_vpn(&mut self) -> VpnStatus {
        let _result = Command::new("openvpn3")
            .arg("session-manage")
            .arg("--disconnect")
            .arg("--config")
            .arg(self.config.clone())
            .output();

        VpnStatus::Disconnected
    }
}

impl VpnConnector for OpenVpn {
    fn connect(&mut self) -> VpnStatus {
        self.connect_open_vpn()
    }

    fn disconnect(&mut self) -> VpnStatus {
        self.disconnect_open_vpn()
    }

    fn status(&mut self) -> VpnStatus {
        self.get_open_vpn_sessions_list()
    }
}
