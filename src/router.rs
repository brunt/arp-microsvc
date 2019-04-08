use actix_web::{
    actix::Addr,
    error, fs,
    http::{header, Method},
    middleware,
    middleware::cors::Cors,
    App,
};
use pnet::datalink::{self, NetworkInterface};
use crate::api::appstate::AppState;
use crate::api::arp::arp_handler;
use std::sync::{Arc, Mutex};

pub fn app_state() -> App<AppState> {
    App::with_state(AppState {
        knowns: Arc::new(Mutex::new(Vec::new())),
        interface: datalink::interfaces()
            .iter()
            .filter(|ip| !ip.is_loopback() && !ip.ips.is_empty())
            .next()
            .unwrap()
            .to_owned(),
    })
    .resource("/arp", |r| r.method(Method::GET).with(arp_handler))
}
