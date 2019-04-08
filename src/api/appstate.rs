use pnet::datalink::NetworkInterface;
use std::sync::{Arc, Mutex};
pub struct AppState{
    //list of memorized mappings from mac to vendor
    //hasmap, or vec of tuples?
    pub knowns: Arc<Mutex<Vec<(String, String)>>>,
    pub interface: NetworkInterface
}

