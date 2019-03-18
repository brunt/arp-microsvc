extern crate pnet;

mod macvendor;

use std::env;

fn main() {
    env::var("MACVENDOR_URL").expect("Missing MacVendor URL");

    //TODO AF
    println!("Hello, world!");
}
