/*
 * na
 *
 * Copyright (C) 2016 Srđan Panić <sp@srdja.me>
 *
 * na is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * na is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with na.  If not, see <http://www.gnu.org/licenses/>.
 */

use get_if_addrs;
use std::cmp::Ordering;
use std::net::IpAddr;

pub fn interface_exists(iface: &str) -> bool {
    if iface == "localhost" {
        return true;
    }
    for i in get_if_addrs::get_if_addrs().unwrap() {
        if i.name == iface {
            return true;
        }
    }
    false
}

pub fn get_iface_addr(iface: &str, ipv6: bool) -> Result<String, String> {
    if iface == "localhost" {
        return if ipv6 {
            Ok("::1".to_string())
        } else {
            Ok("127.0.0.1".to_string())
        };
    }
    let ifaces = get_if_addrs::get_if_addrs().unwrap();
    for i in ifaces {
        if i.name == iface {
            match i.ip() {
                IpAddr::V4(addr) => {
                    if !ipv6 {
                        return Ok(addr.to_string());
                    }
                }
                IpAddr::V6(addr) => {
                    if ipv6 {
                        return Ok(addr.to_string());
                    }
                }
            }
        }
    }
    Err(format!(
        "Could not find a valid address for interface {}",
        iface
    ))
}

/// Returns the first local address
pub fn get_local_addr(ipv6: bool) -> Option<String> {
    let mut ifaces = get_if_addrs::get_if_addrs().unwrap();
    ifaces.sort_by(|a, b| {
        let aip;
        let bip;
        if ipv6 {
            aip = match a.ip() {
                IpAddr::V6(addr) => addr.is_unicast_link_local(),
                _ => false,
            };
            bip = match b.ip() {
                IpAddr::V6(addr) => addr.is_unicast_link_local(),
                _ => false,
            };
        } else {
            aip = match a.ip() {
                IpAddr::V4(addr) => addr.is_private(),
                _ => false,
            };
            bip = match b.ip() {
                IpAddr::V4(addr) => addr.is_private(),
                _ => false,
            };
        }
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
            IpAddr::V4(addr) => Some(addr.to_string()),
            IpAddr::V6(addr) => Some(addr.to_string()),
        };
    }
    None
}

pub fn get_all_addrs() -> Vec<String> {
    let ifaces = get_if_addrs::get_if_addrs().unwrap();
    let mut addrs: Vec<String> = Vec::new();
    for i in ifaces {
        addrs.push(format!("{} @ {}", i.name.clone(), i.ip().to_string()));
    }
    addrs
}
