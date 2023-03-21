use std::io::BufRead;
use std::process::Command;
use crate::vpn::{VpnConnector, Connector, VpnStatus};


pub enum OpenVpnStatsResult {
    Ok(String),
    Err(String)
}

struct OpenVpn;


impl OpenVpn {
    pub fn connect_open_vpn(config: &str) -> VpnStatus {
        let result = Command::new("openvpn3")
            .arg("session-start")
            .arg("--config")
            .arg(config)
            .output();

        return match result {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    let response = String::from_utf8(output.stdout.clone()).unwrap();
                    if response.contains("** ERROR **") {
                        return VpnStatus::Disconnected
                    }
                    return VpnStatus::Connecting
                }
                VpnStatus::Error(String::from_utf8(output.stderr).clone().unwrap())
            }
            Err(_) => {
                VpnStatus::Error(String::from("Unable to connect"))
            }
        }
    }

    pub fn get_open_vpn_stats(config: &str) -> OpenVpnStatsResult {
        let result = Command::new("openvpn3")
            .arg("session-stats")
            .arg("--config")
            .arg(config)
            .output();

        return match result {
            Ok(output) => {
                let response = String::from_utf8(output.stdout).clone().unwrap();
                if response.contains("** ERROR **") {
                    return OpenStatsResult::Err(String::from("Error getting stats"))
                }
                OpenVpnStatsResult::Ok(response)
            }
            Err(_) => {
                OpenVpnStatsResult::Err(String::from("Unable to determine status"))
            }
        }
    }

    pub fn get_open_vpn_sessions_list() -> Status {
        let result = Command::new("openvpn3")
            .arg("sessions-list")
            .output();

        return match result {
            Ok(output) => {
                for line in output.stdout.lines() {
                    match line {
                        Ok(content) => {
                            if content.trim_start().starts_with("Status:") {
                                let status = content.trim().strip_prefix("Status:").unwrap().trim();
                                return match status {
                                    "Web authentication required to connect" => VpnStatus::Authenticating,
                                    "Connection, Client connected" => VpnStatus::Connected,
                                    _ => VpnStatus::Error(format!("Unknown session status, {}", status))
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
                VpnStatus::Error(String::from("Status line not found"))
            }
            Err(_) => {
                VpnStatus::Error(String::from("Unable to get session status"))
            }
        }
    }

    pub fn disconnect_open_vpn(config: &str) {
        let result = Command::new("openvpn3")
            .arg("session-manage")
            .arg("--disconnect")
            .arg("--config")
            .arg(config)
            .output();
    }
}

impl Connector for OpenVpn {
    fn connect(&self) -> VpnStatus {
        let vpn = connect_open_vpn("");
    }

    fn disconnect(&self) -> VpnStatus {
        todo!()
    }

    fn status(&self) -> VpnStatus {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::openvpn::*;

    #[test]
    fn test_connect() {
        const CONFIG: &str = "/home/mdodgson/work/sonatype/config/sonatype.ovpn";

        disconnect_open_vpn(CONFIG);
        assert_eq!(connect_open_vpn(CONFIG), VpnStatus::Connecting);
        assert_eq!(get_open_vpn_sessions_list(), VpnStatus::Authenticating);
        assert_eq!(get_open_vpn_sessions_list(), VpnStatus::Connected);
        let response = get_open_vpn_stats(CONFIG);
        match response {
            OpenStatsResult::Ok(value) => {
                assert!(value.len() > 0)
            }
            OpenStatsResult::Err(_) => { assert!(true)}
        }

        disconnect_open_vpn(CONFIG);
    }
}