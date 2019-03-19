extern crate pnet;

mod macvendor;
mod lib;

use std::{env, thread, time};
use pnet::datalink::{self, Channel, NetworkInterface, MacAddr};


use macvendor::vendor_request;
use lib::{recv_arp_packets, send_arp_packet};
use std::sync::mpsc::{self, Sender, Receiver};
use std::net::{IpAddr, Ipv4Addr};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpOperation, ArpPacket};

use ipnetwork::{IpNetwork};



fn main() {
    //this will all go into lib if it goes well

    //optional variable to add a comma-separated list of known mac addresses to ignore from displaying
    let ignore_list = env::var("IGNORE").unwrap_or_default();
    //the url for the api that returns vendor information from the mac addr
    let vendor_url = env::var("MACVENDOR_URL").expect("Missing MacVendor URL");

    let interface : NetworkInterface = datalink::interfaces()
        .iter()
        .filter(|i| { !i.is_loopback() && !i.ips.is_empty() })
        .next()
        .unwrap()
        .to_owned();
    println!("{}", interface);
//    thread::sleep(time::Duration::from_millis(500));


    let source_mac = interface.mac_address();
    let source_network = interface.ips.first().unwrap();
    let source_ip = source_network.ip();
    let arp_operation = ArpOperations::Request;
    let target_mac = MacAddr::new(255,255,255,255,255,255);

    // Channel for ARP replies.
    let (tx, rx): (Sender<(Ipv4Addr, MacAddr)>, Receiver<(Ipv4Addr, MacAddr)>) = mpsc::channel();

    recv_arp_packets(interface.clone(), tx);

    match source_network {
        //for mac development I had to set ipv6 to manual
        &IpNetwork::V4(source_networkv4) => {
            for target_ipv4 in source_networkv4.iter() {
                match source_ip {
                    IpAddr::V4(source_ipv4) => {
                        send_arp_packet(interface.clone(), source_ipv4, source_mac, target_ipv4, target_mac, arp_operation);
                    },
                    e => panic!("Error while parsing to IPv4 address: {}", e)
                }

            }
        },
//        &IpNetwork::V6(source_networkv6) => {
//            for target_ipv6 in source_networkv6.iter() {
//                match source_ip {
//                    IpAddr::V6(source_ipv6) => {
//                        send_arp_packet(interface.clone(), source_ipv6, source_mac, target_ipv6, target_mac, arp_operation);
//                    },
//                    e => panic!("Error while parsing to IPv4 address: {}", e)
//                }
//
//            }
//        },
        e => panic!("Error while attempting to get network for interface: {}", e)
    }
    thread::sleep(time::Duration::from_millis(500));
    let mut mac_list: Vec<(Ipv4Addr,MacAddr)> = Vec::new();
    loop {
        match rx.try_recv() {
            Ok((ipv4_addr, mac_addr)) => {
                mac_list.push((ipv4_addr, mac_addr))
            },
            Err(_) => break
        }
    }

    for m in mac_list {
//        println!("{}", m);
        println!("{} : {} : {}", m.1, m.0, macvendor::vendor_request(&vendor_url, &m.1.to_string()).unwrap());
        thread::sleep(time::Duration::from_secs(5)); //the api is rate limited
    }
}
