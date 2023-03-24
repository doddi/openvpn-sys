mod dummyvpn;
mod openvpn;
mod vpn;

pub mod prelude {
    pub use crate::{
        create_vpn_connector,
        vpn::{VpnConnector, VpnStatus},
        ConnectorType,
    };
}

use crate::dummyvpn::DummyVpn;
use crate::openvpn::OpenVpn;
use crate::vpn::VpnConnector;

pub enum ConnectorType {
    Dummy,
    Open,
}

pub fn create_vpn_connector(connector_type: ConnectorType) -> Box<dyn VpnConnector + Send> {
    match connector_type {
        ConnectorType::Dummy => Box::new(DummyVpn::new()) as Box<dyn VpnConnector + Send>,
        ConnectorType::Open => Box::new(OpenVpn::new(String::from(
            "/home/mdodgson/work/sonatype/config/sonatype.ovpn",
        ))),
    }
}
