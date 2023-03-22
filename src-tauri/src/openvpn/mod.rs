use std::io::BufRead;
use std::process::{Command, Output};
use tauri::utils::debug_eprintln;
use crate::vpn::{VpnConnector, VpnStatus};


enum OpenVpnStatsResult {
    Ok(String),
    Err(String)
}

#[derive(Clone, serde::Serialize)]
pub struct OpenVpn {
    config: String,
    status: VpnStatus
}

// TODO Wrap Command to allow testing
impl OpenVpn {
    pub fn new(config: String) -> Self {
        OpenVpn { config, status: VpnStatus::Disconnected }
    }

    fn connect_open_vpn(&mut self) -> VpnStatus {
        let result = Command::new("openvpn3")
            .arg("session-start")
            .arg("--config")
            .arg(self.config.clone())
            .output();

        self.status = match result {
            Ok(output) => Self::check_connect_response(output),
            Err(_) => VpnStatus::Error(String::from("Unable to connect"))
        };
        self.status.clone()
    }

    fn check_connect_response(output: Output) -> VpnStatus {
        if !output.stdout.is_empty() {
            let response = String::from_utf8(output.stdout.clone()).unwrap();
            if response.contains("** ERROR **") {
                return VpnStatus::Disconnected
            }
            return VpnStatus::Connecting
        }
        return VpnStatus::Error(String::from_utf8(output.stderr).clone().unwrap())
    }

    fn get_open_vpn_stats(&self) -> OpenVpnStatsResult {
        let result = Command::new("openvpn3")
            .arg("session-stats")
            .arg("--config")
            .arg(self.config.clone())
            .output();

        return match result {
            Ok(output) => {
                let response = String::from_utf8(output.stdout).clone().unwrap();
                if response.contains("** ERROR **") {
                    return OpenVpnStatsResult::Err(String::from("Error getting stats"))
                };
                OpenVpnStatsResult::Ok(response)
            }
            Err(_) => {
                OpenVpnStatsResult::Err(String::from("Unable to determine status"))
            }
        }
    }

    fn determine_status(output: Output) -> VpnStatus {
        debug_eprintln!("determine_status: {}", String::from_utf8(output.stdout.clone()).unwrap());
        for line in output.stdout.lines() {
            match line {
                Ok(content) => {
                    if content.trim_start().starts_with("Status:") {
                        let status = content.trim().strip_prefix("Status:").unwrap().trim();
                        return match status {
                            "Web authentication required to connect" => VpnStatus::Authenticating,
                            "Connection, Client connected" => VpnStatus::Connected,
                            _ => VpnStatus::Error(format!("Unknown session status, {}", status))
                        };
                    }
                    else if content.trim().contains("No sessions available") {
                        return VpnStatus::Disconnected
                    }
                }
                Err(_) => {}
            }
        }
        VpnStatus::Error(String::from("Status line not found"))
    }

    fn get_open_vpn_sessions_list(&mut self) -> VpnStatus {
        let result = Command::new("openvpn3")
            .arg("sessions-list")
            .output();

        self.status = match result {
            Ok(output) => Self::determine_status(output),
            Err(_) => VpnStatus::Error(String::from("Unable to get session status"))
        };
        debug_eprintln!("get_open_vpn_sessions_list: {:?}", self.status);
        self.status.clone()
    }

    fn disconnect_open_vpn(&mut self) -> VpnStatus {
        let result = Command::new("openvpn3")
            .arg("session-manage")
            .arg("--disconnect")
            .arg("--config")
            .arg(self.config.clone())
            .output();

        self.status = VpnStatus::Disconnected;
        self.status.clone()
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

#[cfg(test)]
mod tests {
    use crate::openvpn::*;

    #[test]
    fn test_connect() {
        const CONFIG: String = "/home/mdodgson/work/sonatype/config/sonatype.ovpn".into_string();

        let mut open_vpn = OpenVpn::new(CONFIG);
        assert_eq!(open_vpn.disconnect_open_vpn(), VpnStatus::Disconnected);
        assert_eq!(open_vpn.connect_open_vpn(), VpnStatus::Connecting);
        assert_eq!(open_vpn.get_open_vpn_sessions_list(), VpnStatus::Authenticating);
        assert_eq!(open_vpn.get_open_vpn_sessions_list(), VpnStatus::Connected);
        let response = open_vpn.get_open_vpn_stats();
        match response {
            OpenVpnStatsResult::Ok(value) => {
                assert!(value.len() > 0)
            }
            OpenVpnStatsResult::Err(_) => { assert!(true)}
        }

        open_vpn.disconnect_open_vpn();
    }
}