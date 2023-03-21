use std::io::BufRead;
use std::process::Command;
use crate::vpn::{Connector, VpnStatus};


enum OpenVpnStatsResult {
    Ok(String),
    Err(String)
}

pub struct OpenVpn<'a> {
    config: &'a str,
    status: VpnStatus
}

impl OpenVpn {
    pub fn new(config: &str) -> Self {
        OpenVpn { config, status: VpnStatus::Disconnected }
    }

    fn connect_open_vpn(&mut self, config: &str) -> &VpnStatus {
        let result = Command::new("openvpn3")
            .arg("session-start")
            .arg("--config")
            .arg(config)
            .output();

        self.status =  match result {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    let response = String::from_utf8(output.stdout.clone()).unwrap();
                    if response.contains("** ERROR **") {
                        VpnStatus::Disconnected
                    }
                    VpnStatus::Connecting
                }
                VpnStatus::Error(String::from_utf8(output.stderr).clone().unwrap())
            }
            Err(_) => {
                VpnStatus::Error(String::from("Unable to connect"))
            }
        };
        return &self.status
    }

    fn get_open_vpn_stats(config: &str) -> OpenVpnStatsResult {
        let result = Command::new("openvpn3")
            .arg("session-stats")
            .arg("--config")
            .arg(config)
            .output();

        return match result {
            Ok(output) => {
                let response = String::from_utf8(output.stdout).clone().unwrap();
                if response.contains("** ERROR **") {
                    OpenVpnStatsResult::Err(String::from("Error getting stats"))
                }
                OpenVpnStatsResult::Ok(response)
            }
            Err(_) => {
                OpenVpnStatsResult::Err(String::from("Unable to determine status"))
            }
        }
    }

    fn get_open_vpn_sessions_list(&mut self) -> &VpnStatus {
        let result = Command::new("openvpn3")
            .arg("sessions-list")
            .output();

        self.status = match result {
            Ok(output) => {
                for line in output.stdout.lines() {
                    match line {
                        Ok(content) => {
                            if content.trim_start().starts_with("Status:") {
                                let status = content.trim().strip_prefix("Status:").unwrap().trim();
                                match status {
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
        };
        &self.status
    }

    fn disconnect_open_vpn(&mut self, config: &str) -> &VpnStatus {
        let result = Command::new("openvpn3")
            .arg("session-manage")
            .arg("--disconnect")
            .arg("--config")
            .arg(config)
            .output();

        self.status = VpnStatus::Disconnected;
        &self.status
    }
}

impl Connector for OpenVpn {
    fn connect(&mut self) -> &VpnStatus {
        self.connect_open_vpn(self.config)
    }

    fn disconnect(&mut self) -> &VpnStatus {
        self.disconnect_open_vpn(self.config)
    }

    fn status(&mut self) -> &VpnStatus {
        self.get_open_vpn_sessions_list()
    }
}

#[cfg(test)]
mod tests {
    use crate::openvpn::*;

    #[test]
    fn test_connect() {
        const CONFIG: &str = "/home/mdodgson/work/sonatype/config/sonatype.ovpn";

        let mut open_vpn = OpenVpn::new();
        assert_eq!(open_vpn.disconnect_open_vpn(CONFIG), VpnStatus::Disconnected);
        assert_eq!(open_vpn.connect_open_vpn(CONFIG), VpnStatus::Connecting);
        assert_eq!(open_vpn.get_open_vpn_sessions_list(), VpnStatus::Authenticating);
        assert_eq!(open_vpn.get_open_vpn_sessions_list(), VpnStatus::Connected);
        let response = open_vpn.get_open_vpn_stats(CONFIG);
        match response {
            OpenVpnStatsResult::Ok(value) => {
                assert!(value.len() > 0)
            }
            OpenVpnStatsResult::Err(_) => { assert!(true)}
        }

        open_vpn.disconnect_open_vpn(CONFIG);
    }
}