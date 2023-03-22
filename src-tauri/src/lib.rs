mod openvpn;
mod vpn;
mod dummyvpn;

pub mod prelude {
    pub use crate::{
        vpn::{VpnConnector, VpnStatus},
        create_vpn_connector, ConnectorType
    };
}

use crate::dummyvpn::DummyVpn;
use crate::openvpn::OpenVpn;
use crate::vpn::VpnConnector;


pub enum ConnectorType {
    Dummy,
    Open
}

pub fn create_vpn_connector(connector_type: ConnectorType) -> Box<dyn VpnConnector> {
    return match connector_type {
        ConnectorType::Dummy => Box::new(DummyVpn::new()) as Box<dyn VpnConnector + Send>,
        ConnectorType::Open => Box::new(OpenVpn::new(String::from("/home/mdodgson/work/sonatype/config/sonatype.ovpn")))
    };
}