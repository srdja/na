
use std::mem;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::net::IpAddr;
use libc::{self, c_int};
use get_if_addrs;
use get_if_addrs::Interface;

/// Note: Taken from the unstable stdlib release
///
/// Returns true if this is a private address.
///
/// The private address ranges are defined in RFC1918 and include:
///
///  - 10.0.0.0/8
///  - 172.16.0.0/12
///  - 192.168.0.0/16
///
pub fn is_private(ip: Ipv4Addr) -> bool {
    match (ip.octets()[0], ip.octets()[1]) {
        (10, _) => true,
        (172, b) if b >= 16 && b <= 31 => true,
        (192, 168) => true,
        _ => false
    }
}


/// Converts an integer representation of ipv4 address to Ipv4Addr
pub fn to_ipaddr(addr: libc::c_int) -> Ipv4Addr {
    Ipv4Addr::new( addr        as u8,
                  (addr >> 8)  as u8,
                  (addr >> 16) as u8,
                  (addr >> 24) as u8)
}

#[cfg(target_os = "linux")]
pub fn get_local_addresses() -> Option<Vec<String>> {
    let mut addresses: Vec<String> = Vec::new();

    unsafe {
        let mut next: *mut libc::ifaddrs = mem::zeroed();
        let status = libc::getifaddrs(&mut next);

        if status == -1 {
            return None;
        }
        while !next.is_null() {
            let address: *mut libc::sockaddr_in = mem::transmute((*next).ifa_addr);
            if (*address).sin_family == (libc::AF_INET as u16) {
                let addr = to_ipaddr((*address).sin_addr.s_addr as c_int);
                if is_private(addr) {
                    addresses.push(addr.to_string());
                }
            }
            next = (*next).ifa_next;
        }
    }
    Some(addresses)
}


#[cfg(target_os = "windows")]
pub fn get_local_addresses() -> Option<Vec<String>> {
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
    Some(addresses)
}
