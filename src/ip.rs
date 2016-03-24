
use std::mem;
use std::net::Ipv4Addr;
use libc::c_int;

use libc::getifaddrs;
use libc::ifaddrs;
use libc::sockaddr_in;
use libc::AF_INET;


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
pub fn to_ipaddr(addr: c_int) -> Ipv4Addr {
    Ipv4Addr::new( addr        as u8,
                  (addr >> 8)  as u8,
                  (addr >> 16) as u8,
                  (addr >> 24) as u8)
}


pub fn get_local_addresses() -> Option<Vec<String>> {
    let mut addresses: Vec<String> = Vec::new();

    unsafe {
        let mut next: *mut ifaddrs = mem::zeroed();
        let status = getifaddrs(&mut next);

        if status == -1 {
            return None;
        }
        while !next.is_null() {
            let address: *mut sockaddr_in = mem::transmute((*next).ifa_addr);
            if (*address).sin_family == (AF_INET as u16) {
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
