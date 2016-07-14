/*
 * na
 *
 * Copyright (C) 2016 Srđan Panić <i@srdja.me>
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


extern crate getopts;
extern crate hyper;
extern crate mime;
extern crate regex;
extern crate url;
extern crate mustache;
extern crate get_if_addrs;
extern crate multipart;


macro_rules! println_cond {
    ($b:expr, $s:expr, $($p:expr),*) => (
        if $b {println!($s, $($p,)*)})
}


mod ip;
mod directory;
mod request;
mod template;
mod static_r;

use getopts::Options;
use hyper::server::{Handler, Server};
use directory::Directory;
use request::RequestHandler;
use static_r::Resource;

use std::path::PathBuf;
use std::env;


const VERSION: &'static str = "0.0.1";


fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help message");
    opts.optopt("d", "dir", "Path of the served directory. Working
directory is served by default if none is pecified.", "PATH");
    opts.optopt("p", "port", "Port number", "NUMBER");
    opts.optflag("o", "overwrite-file", "If set, uploaded files will overwrite existing files with the same name.");
    opts.optopt("i", "interface", "Specify an interface to use", "INTERFACE");
    opts.optflag("l", "list-interfaces", "Print a list of available interfaces");
    opts.optflag("v", "verbose", "Verbose output");
    opts.optflag("", "version", "Print version info");

    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(e) => panic!(e.to_string())
    };

    if matches.opt_present("version") {
        print_version_info();
        return;
    }

    if matches.opt_present("l") {
        for i in ip::get_all_addrs() {
            println!("{}", i);
        }
        return;
    }

    if matches.opt_present("h") {
        print_help(&program_name, opts);
        return;
    }

    let current_dir = match matches.opt_str("d") {
        Some(d) => {
            let mut dir = PathBuf::new();
            dir.push(d);
            dir
        },
        None => env::current_dir().unwrap()
    };

    let port = match matches.opt_str("p") {
        Some(p) => p,
        None    => "9000".to_string()
    };

    let address = match matches.opt_str("i") {
        Some(a) => {
            if ip::interface_exists(a.clone()) {
                match ip::get_iface_addr(a) {
                    Ok(i)  => Some(i),
                    Err(e) => {
                        println!("{}", e);
                        None
                    }
                }
            } else {
                panic!("Error: Specified interface \"{}\" does not exist", a);
            }
        },
        None => {
            ip::get_local_addr()
        }
    };

    let addr_and_port = match address {
        Some(a) => {
            format!("{}:{}", a, port)
        },
        None => {
            panic!("Error: No active network interfaces found!");
        },
    };

    println!("Serving contents of {} at http://{}", current_dir.to_str().unwrap(), addr_and_port);

    let directory   = Directory::new(current_dir);
    let static_res  = Resource::new();
    let req_handler = RequestHandler::new(directory, static_res, matches.opt_present("v"));


    Server::http(&*addr_and_port).unwrap()
            .handle(req_handler).unwrap();
}


fn print_help(name: &str, opts: Options) {
    let brief = format!("Usage: {} [OPTIONS]", name);
    println!("{}", opts.usage(&brief));
}


fn print_version_info() {
    println!("na {}
Copyright (C) 2016 Srđan Panić <i@srdja.me>.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.", VERSION);
}
