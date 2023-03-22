#[derive(Clone, Debug, serde::Serialize)]
pub enum VpnStatus {
    Connected,
    Connecting,
    Authenticating,
    Disconnected,
    Error(String)
}

pub trait VpnConnector {
    fn connect(&mut self) -> &VpnStatus;
    fn disconnect(&mut self) -> &VpnStatus;
    fn status(&mut self) -> &VpnStatus;
}
