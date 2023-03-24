use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Debug, serde::Serialize)]
pub enum VpnStatus {
    Disconnected,
    Initialising,
    Connecting,
    Authenticating,
    Connected,
    Disconnecting,
    Error(String),
}

impl fmt::Display for VpnStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            VpnStatus::Disconnected => {
                write!(f, "Disconnected")
            }
            VpnStatus::Initialising => {
                write!(f, "Initialising")
            }
            VpnStatus::Connecting => {
                write!(f, "Connecting")
            }
            VpnStatus::Authenticating => {
                write!(f, "Authenticating")
            }
            VpnStatus::Connected => {
                write!(f, "Connected")
            }
            VpnStatus::Disconnecting => {
                write!(f, "Disconnecting")
            }
            VpnStatus::Error(s) => {
                write!(f, "Error, {}", s)
            }
        }
    }
}

pub trait VpnConnector {
    fn connect(&mut self) -> VpnStatus;
    fn disconnect(&mut self) -> VpnStatus;
    fn status(&mut self) -> VpnStatus;
}
