use std::net::IpAddr;
use get_if_addrs;
use get_if_addrs::Interface;


pub fn get_local_addresses() -> Vec<String> {
    let mut addresses: Vec<String> = Vec::new();
    let interfaces: Vec<Interface> = get_if_addrs::get_if_addrs().unwrap();

    for i in interfaces {
        match i.ip() {
            IpAddr::V4(addr) => {
                if addr.is_private() {
                    addresses.push(addr.to_string());
                }
            },
            _ => {},
        }
    }
    addresses
}
