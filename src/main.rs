extern crate pnet;

use std::{env, thread, time};
use pnet::datalink::{self, Channel, NetworkInterface, MacAddr};


use lib::macvendor::vendor_request;
use lib::arp::{arp_results};
use std::sync::mpsc::{self, Sender, Receiver};
use std::net::{IpAddr, Ipv4Addr};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpOperation, ArpPacket};

use ipnetwork::{IpNetwork};

mod lib;

fn main() {
    arp_results();
}
