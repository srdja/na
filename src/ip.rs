
use std::mem;
use std::net::Ipv4Addr;
use libc::c_int;
use libc::size_t;


extern {
    fn get_local_addr(addr: *const i32, max_addrs: size_t) -> c_int;
}


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
    let buff: [c_int; 5];
    let addrs;

    unsafe {
        buff  = mem::zeroed();
        addrs = get_local_addr(mem::transmute(&(buff[0])), 5);
    }
    if addrs == 0 {
        return None;
    }
    for a in 0..addrs {
        let addr = to_ipaddr(buff[a as usize]);
        if is_private(addr) {
            addresses.push(to_ipaddr(buff[a as usize]).to_string());
        }
    }
    Some(addresses)
}
