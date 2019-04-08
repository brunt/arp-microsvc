extern crate pnet;

use actix_web::{actix::System, server};
use pnet::datalink::{self, Channel, MacAddr, NetworkInterface};
use std::{env, thread, time};

use api::arp::arp_results;
use api::macvendor::vendor_request;
use pnet::packet::arp::{ArpHardwareTypes, ArpOperation, ArpOperations, ArpPacket};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{self, Receiver, Sender};

use ipnetwork::IpNetwork;

mod api;
mod router;

fn main() {
    let port = env::var("PORT").unwrap_or("8000".to_string());

    let sys = System::new("arp-microsvc");
    server::new(move || router::app_state())
        .bind(format!("localhost:{}", &port))
        .unwrap()
        .shutdown_timeout(2)
        .start();

    sys.run();
    println!("app started on port {}", port);
}
