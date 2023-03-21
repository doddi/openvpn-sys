#[derive(PartialEq, Debug)]
pub enum VpnStatus {
    Connected,
    Connecting,
    Authenticating,
    Disconnected,
    Error(String)
}

pub trait Connector {
    fn connect(&mut self) -> &VpnStatus;
    fn disconnect(&mut self) -> &VpnStatus;
    fn status(&mut self) -> &VpnStatus;
}
