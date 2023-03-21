pub struct VpnConnector {
    status: Status
}

#[derive(PartialEq, Debug)]
pub enum VpnStatus {
    Connected,
    Connecting,
    Authenticating,
    Disconnected,
    Error(String)
}

pub trait Connector {
    fn connect(&self) -> VpnStatus;
    fn disconnect(&self) -> VpnStatus;
    fn status(&self) -> VpnStatus;
}
