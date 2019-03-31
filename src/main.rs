extern crate pnet;

use std::{env, thread, time};
use pnet::datalink::{self, Channel, NetworkInterface, MacAddr};


use api::macvendor::vendor_request;
use api::arp::{arp_results};
use std::sync::mpsc::{self, Sender, Receiver};
use std::net::{IpAddr, Ipv4Addr};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpOperation, ArpPacket};

use ipnetwork::{IpNetwork};

mod api;

fn main() {
    arp_results();
}
