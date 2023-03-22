use crate::vpn::{VpnConnector, VpnStatus};

#[derive(Clone, serde::Serialize)]
pub struct DummyVpn;

impl VpnConnector for DummyVpn {
    fn connect(&mut self) -> &VpnStatus {
        &VpnStatus::Connecting
    }

    fn disconnect(&mut self) -> &VpnStatus {
        &VpnStatus::Disconnected
    }

    fn status(&mut self) -> &VpnStatus {
        &VpnStatus::Disconnected
    }
}

impl DummyVpn {
    pub fn new() -> DummyVpn { DummyVpn {} }
}
