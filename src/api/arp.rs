use actix_web::{HttpRequest, HttpResponse};
use pnet::datalink::{self, Channel, MacAddr, NetworkInterface};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{self, Receiver, Sender, SendError};
use std::{env, error::Error, thread, time};

use pnet::packet::arp::MutableArpPacket;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::{MutablePacket, Packet};

use pnet::packet::arp::{ArpHardwareTypes, ArpOperation, ArpOperations, ArpPacket};

use crate::api::macvendor::vendor_request;
use crate::api::models::{AppState, ArpResponse, ArpResponses};
use ipnetwork::IpNetwork;

pub fn send_arp_packet(
    interface: NetworkInterface,
    source_ip: Ipv4Addr,
    source_mac: MacAddr,
    target_ip: Ipv4Addr,
) {
    let (mut tx, _) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };
    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

    ethernet_packet.set_destination(MacAddr::broadcast());
    ethernet_packet.set_source(source_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(source_mac);
    arp_packet.set_sender_proto_addr(source_ip);
    arp_packet.set_target_hw_addr(MacAddr::zero());
    arp_packet.set_target_proto_addr(target_ip);

    ethernet_packet.set_payload(arp_packet.packet_mut());

    tx.send_to(ethernet_packet.packet(), Some(interface));
}

pub fn recv_arp_packets(interface: NetworkInterface, tx: Sender<MacAddr>) {
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
                        let result: MacAddr = arp_packet.get_sender_hw_addr();
                        match tx.send(result) {
                            Ok(()) => (),
                            Err(SendError(e)) => {

                                dbg!(e);
                                ()
                            }
                        }
                    }
                }
                Err(e) => panic!("An error occurred while reading packet: {}", e),
            }
        }
    });
}

//set up a channel, call send and recv
pub fn arp_results(
    interface: NetworkInterface,
    knowns: &mut ArpResponses,
) -> Result<ArpResponses, Box<Error>> {

    //optional variable to add a comma-separated list of known mac addresses to ignore from displaying
    let ignores = env::var("IGNORE").unwrap_or_default();
    let ignores_vec: Vec<&str> = ignores.split(",").collect();

    //the url for the api that returns vendor information from the mac addr
    let vendor_url = env::var("MACVENDOR_URL").unwrap_or("https://api.macvendors.com".to_string());
    let source_mac = interface.mac_address();
    let source_network = interface.ips.iter().find(|ip| ip.is_ipv4()).unwrap();
    let source_ip = source_network.ip();

    // Channel for ARP replies.
    let (tx, rx): (Sender<MacAddr>, Receiver<MacAddr>) = mpsc::channel();
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
                        );
                    }
                    e => {
                        println!("Error while parsing to IPv4 address: {}", e);
                    }
                }
            }
        },
        e => {
            println!("Error while attempting to get network for interface: {}", e);
        }
    }
    let mut mac_list: Vec<MacAddr> = Vec::new();
    loop {
        match rx.try_recv() {
            Ok( mac_addr) => mac_list.push(mac_addr),
            Err(_) => break,
        }
    }
    let mut output = ArpResponses {
        results: Vec::new(),
    };
    for m in mac_list {
        let short_mac = &m.to_string()[..8]; //only the first 6 hex characters are required to obtain vendor name and compare.
        if !ignores_vec.contains(&short_mac) && !knowns.results.contains(&ArpResponse{
            mac_addr: short_mac.to_string(),
            vendor_name: "".to_string(),
        }) {
            //mac addr -> String -> &str
            match vendor_request(&vendor_url, short_mac) {
                Ok(s) => {
                    output.results.push(ArpResponse {
                        mac_addr: short_mac.to_string(),
                        vendor_name: s.clone(),
                    });
                    knowns.results.push(ArpResponse{
                        mac_addr: short_mac.to_string(),
                        vendor_name: s,
                    });
                    println!("{:?}",knowns);
                }
                Err(e) => {
                    // send error, channel was closed too soon
                    // an error here means not all responses are shown
                    println!("{:?}", e);
                }
            }
            thread::sleep(time::Duration::from_secs(1)); //the api is rate limited so pause between calls
        }
    }
    Ok(output)
}

//get interface and knowns from appstate,
//read them both and pass them both into arp_results?
pub fn arp_handler(req: HttpRequest<AppState>) -> HttpResponse {
    let iface = req.state().interface.clone();
    match req.state().knowns.lock() {
        Ok(mut k) => {
            //read list of knowns,
            //if a mac addr on local network is not in list of knowns, call vendor api, then store results from api back into knowns
            match arp_results(iface, &mut k) {
                Ok(response) => HttpResponse::Ok().json(response),
                Err(_) => HttpResponse::InternalServerError().finish()
            }
        }
        Err(e) => {
            println!("error obtaining mutex lock: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
