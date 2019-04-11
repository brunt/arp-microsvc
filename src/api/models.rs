use pnet::datalink::NetworkInterface;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize)]
pub struct ArpResponse {
    pub mac_addr: String,
    pub vendor_name: String,
}

#[derive(Debug, Serialize)]
pub struct ArpResponses {
    pub results: Vec<ArpResponse>,
}

pub struct AppState {
    pub knowns: Arc<Mutex<ArpResponses>>,
    pub interface: NetworkInterface,
}

impl PartialEq for ArpResponse {
    fn eq(&self, other: &ArpResponse) -> bool {
        self.mac_addr == other.mac_addr
    }
}
