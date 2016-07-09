use std::net::IpAddr;
use get_if_addrs;
use std::cmp::Ordering;


pub fn interface_exists(iface: String) -> bool {
    for i in get_if_addrs::get_if_addrs().unwrap() {
        if i.name == iface {
            return true;
        }
    }
    false
}


pub fn get_iface_addr(iface: String) -> Result<String, String> {
    let ifaces = get_if_addrs::get_if_addrs().unwrap();
    for i in ifaces {
        if i.name == iface {
            match i.ip() {
                IpAddr::V4(addr) => {
                    return Ok(addr.to_string());
                },
                IpAddr::V6(addr) => {}
            }
        }
    }
    Err(format!("Could not find a valid address for interface {}", iface))
}


/// Returns the first local address
pub fn get_local_addr() -> Option<String> {
    let mut ifaces = get_if_addrs::get_if_addrs().unwrap();
    ifaces.sort_by(|a, b| {
        let aip = match a.ip() {IpAddr::V4(addr) => addr.is_private(), _ => false};
        let bip = match b.ip() {IpAddr::V4(addr) => addr.is_private(), _ => false};
        if aip && !bip {
            Ordering::Less
        } else if aip && bip {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    });
    if ifaces.len() > 0 {
        return match ifaces[0].ip() {
            IpAddr::V4(addr) => {
                Some(addr.to_string())
            },
            IpAddr::V6(addr) => {
                Some(addr.to_string())
            }
        }
    }
    None
}


pub fn get_all_addrs() -> Vec<String> {
    let ifaces = get_if_addrs::get_if_addrs().unwrap();
    let mut addrs: Vec<String>  = Vec::new();
    for i in ifaces {
        addrs.push(format!("{} @ {}", i.name.clone(), i.ip().to_string()));
    }
    addrs
}
