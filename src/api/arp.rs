use actix_web::{FutureResponse, HttpRequest, HttpResponse, Json, State};
use pnet::datalink::{self, Channel, MacAddr, NetworkInterface};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{self, Receiver, Sender};
use std::{env, thread, time};

use pnet::packet::arp::MutableArpPacket;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::{MutablePacket, Packet};

use pnet::packet::arp::{ArpHardwareTypes, ArpOperation, ArpOperations, ArpPacket};

use crate::api::macvendor::vendor_request;
use crate::api::appstate::AppState;
use ipnetwork::IpNetwork;

pub fn send_arp_packet(
    interface: NetworkInterface,
    source_ip: Ipv4Addr,
    source_mac: MacAddr,
    target_ip: Ipv4Addr,
    target_mac: MacAddr,
    arp_operation: ArpOperation,
) {
    let (mut tx, _) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };

    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

    ethernet_packet.set_destination(target_mac);
    ethernet_packet.set_source(source_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4); //change this to use ipv6
    arp_packet.set_operation(arp_operation);
    arp_packet.set_sender_hw_addr(source_mac);
    arp_packet.set_sender_proto_addr(source_ip);
    arp_packet.set_target_hw_addr(target_mac);
    arp_packet.set_target_proto_addr(target_ip);

    ethernet_packet.set_payload(arp_packet.packet_mut());

    tx.send_to(ethernet_packet.packet(), Some(interface));
}

pub fn recv_arp_packets(interface: NetworkInterface, tx: Sender<(Ipv4Addr, MacAddr)>) {
    thread::spawn(move || {
        let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("Unknown channel type"),
            Err(e) => panic!("Error happened {}", e),
        };

        loop {
            match rx.next() {
                Ok(data) => {
                    let ethernet_packet = EthernetPacket::new(data).unwrap();
                    let ethernet_payload = ethernet_packet.payload();
                    let arp_packet = ArpPacket::new(ethernet_payload).unwrap();
                    let arp_reply_op = ArpOperation::new(2_u16);

                    if arp_packet.get_operation() == arp_reply_op {
                        let result: (Ipv4Addr, MacAddr) = (
                            arp_packet.get_sender_proto_addr(),
                            arp_packet.get_sender_hw_addr(),
                        );
                        match tx.send(result) {
                            Ok(()) => (),
                            Err(e) => {
//                                println!("{:?}",e);
                                dbg!(e);
                                ()
                            }
                        }
//                        tx.send(result).unwrap();
                    }
                }
                Err(e) => panic!("An error occurred while reading packet: {}", e),
            }
        }
    });
}

//set up a channel, call send and recv, return results as a vec?
pub fn arp_results(interface: NetworkInterface) -> Vec<String> {
    //pub fn arp_results(req: HttpRequest<AppState>) -> FutureResponse<HttpResonse> {
    //optional variable to add a comma-separated list of known mac addresses to ignore from displaying
    let ignore_list = env::var("IGNORE").unwrap_or_default(); //todo: comma separated macs in env parsed here
    let ignores_vec: Vec<&str> = ignore_list.split(",").collect();
    //the url for the api that returns vendor information from the mac addr
    let vendor_url = env::var("MACVENDOR_URL").unwrap_or("https://api.macvendors.com".to_string());
    let source_mac = interface.mac_address();
    let source_network = interface.ips.iter().find(|ip| ip.is_ipv4()).unwrap();
    let source_ip = source_network.ip();
    let arp_operation = ArpOperations::Request;
    let target_mac = MacAddr::new(255, 255, 255, 255, 255, 255);

    // Channel for ARP replies.
    let (tx, rx): (Sender<(Ipv4Addr, MacAddr)>, Receiver<(Ipv4Addr, MacAddr)>) = mpsc::channel();

    recv_arp_packets(interface.clone(), tx);

    match source_network {
        //for mac development I had to set ipv6 to manual
        &IpNetwork::V4(source_networkv4) => {
            for target_ipv4 in source_networkv4.iter() {
                match source_ip {
                    IpAddr::V4(source_ipv4) => {
                        send_arp_packet(
                            interface.clone(),
                            source_ipv4,
                            source_mac,
                            target_ipv4,
                            target_mac,
                            arp_operation,
                        );
                    }
                    e => panic!("Error while parsing to IPv4 address: {}", e),
                }
            }
        }
        e => panic!("Error while attempting to get network for interface: {}", e),
    }
    thread::sleep(time::Duration::from_secs(5));
    let mut mac_list: Vec<(Ipv4Addr, MacAddr)> = Vec::new();
    loop {
        match rx.try_recv() {
            Ok((ipv4_addr, mac_addr)) => mac_list.push((ipv4_addr, mac_addr)),
            Err(_) => break,
        }
    }
    let mut output: Vec<String> = Vec::new();
    for m in mac_list {
        if !ignores_vec.contains(&&format!("{}", m.1)[..]) {
            //is there a direct way from mac to &str?
            match vendor_request(&vendor_url, &m.1.to_string()[..8]) {
                Ok(s) => {
                    output.push(s);
                },
                Err(e) => {
                    println!("{:?}", e);
                    //send error, channel was closed too soon
                }
            }
            thread::sleep(time::Duration::from_secs(1)); //the api is rate limited
        }
    }
    output
}

//todo: try futureresponse, error handling
pub fn arp_handler(req: HttpRequest<AppState>) -> HttpResponse {
    let iface = req.state().interface.clone();
    let mut knowns = req.state().knowns.lock();
    let outvec = arp_results(iface);

    HttpResponse::Ok().json(outvec)
}
