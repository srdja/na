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


#![feature(ip)]


extern crate getopts;
extern crate hyper;
extern crate mime;
extern crate regex;
extern crate url;
extern crate mustache;
extern crate get_if_addrs;
extern crate multipart;
extern crate hyper_router;
extern crate chrono;
extern crate rustc_serialize;


macro_rules! println_cond {
    ($b:expr, $($p:expr),+) => (
        if $b {println!($($p,)+)})
}


macro_rules! printerr_cond {
    ($b:expr, $($p:expr),+) => (
        if $b {
            use std::io::Write;
            match writeln!(&mut ::std::io::stderr(), $($p,)+) {
                Ok(_)  => {},
                Err(e) => {panic!("Write to stderr failed: {}", e);}
            };
        })
}


mod ip;
mod directory;
mod routes;
mod format;
mod static_r;

use getopts::Options;
use hyper::server::Server;
use hyper_router::{RouterBuilder, Route};
use directory::Directory;

use routes::{HandlerState,
             IndexHandler,
             FileDownloadHandler,
             ListHandler,
             DeleteHandler,
             FileUploadHandler,
             JSONHandler,
             StaticResourceHandler};

use static_r::Resource;

use std::sync::Arc;
use std::path::PathBuf;
use std::env;


const VERSION: &'static str = "0.1.0";


fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help",
                 "Print this help message");
    opts.optopt("d", "dir",
                "Path of the served directory. Working \
                 directory is served by default if none \
                 is pecified.", "PATH");
    opts.optflag("r", "allow-remove",
                 "If set, na will allow file deletions \
                  trough DELETE requests");
    opts.optopt("p", "port",
                "Port number", "NUMBER");
    opts.optflag("o", "overwrite-file",
                 "If set, uploaded files will\
                  overwrite existing files with\
                  the same name.");
    opts.optflag("s", "show-directory",
                 "If set, the name of the served \
                  directory will be displayed on \
                  html page");
    opts.optopt("i", "interface",
                "Specify an interface to use (eg. \"eth0\", \
                 \"wlo0\", \"localhost\")", "INTERFACE");
    opts.optflag("6", "ipv6",
                 "Use ipv6 if available");
    opts.optflag("l", "list-interfaces",
                 "Print a list of available interfaces");
    opts.optflag("v", "verbose",
                 "Verbose output");
    opts.optflag("", "version",
                 "Print version info");

    let options = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(e) => {
            printerr_cond!(true, "Error: {}", e);
            return;
        }
    };

    if options.opt_present("version") {
        print_version_info();
        return;
    }
    if options.opt_present("l") {
        for i in ip::get_all_addrs() {
            println!("{}", i);
        }
        return;
    }
    if options.opt_present("h") {
        print_help(&program_name, opts);
        return;
    }

    let current_dir = match options.opt_str("d") {
        Some(d) => {
            let mut dir = PathBuf::new();
            dir.push(d);
            dir
        },
        None => env::current_dir().unwrap()
    };
    let port = match options.opt_str("p") {
        Some(p) => p,
        None    => "9000".to_string()
    };
    let address = match options.opt_str("i") {
        Some(a) => {
            if ip::interface_exists(&a) {
                match ip::get_iface_addr(&a, options.opt_present("6")) {
                    Ok(i)  => Some(i),
                    Err(e) => {
                        println!("{}", e);
                        None
                    }
                }
            } else {
                printerr_cond!(true, "Error: Specified interface \
                                      \"{}\" does not exist!", a);
                return;
            }
        },
        None => {
            ip::get_local_addr(options.opt_present("6"))
        }
    };
    let addr = match address {
        Some(a) => a,
        None => {
            printerr_cond!(true, "Error: No active network interfaces found!");
            return;
        }
    };

    let str_path   = current_dir.to_str().unwrap().clone().to_string();
    let directory  = Directory::new(current_dir);
    let static_res = Resource::new();

    let addr_and_port = format!("{}:{}", addr, port);
    let srv = match Server::http(&*addr_and_port) {
        Ok(s)  => s,
        Err(e) => {
            printerr_cond!(true, "Error: Unable to start na at ({}), {}",
                           addr_and_port, e);
            return;
        }
    };

    if options.opt_present("6") {
        println!("Serving contents of {} at http://[{}]:{}", str_path, addr, port);
    } else {
        println!("Serving contents of {} at http://{}", str_path, addr_and_port);
    }

    let hs = Arc::new(HandlerState {
        directory: directory,
        resource:  static_res,
        verbose:   options.opt_present("v"),
        delete:    options.opt_present("r"),
        showdir:   options.opt_present("s"),
        overwrite: options.opt_present("o"),
        path:      str_path.clone()
    });

    let index_handler    = IndexHandler(hs.clone());
    let file_dl_handler  = FileDownloadHandler(hs.clone());
    let file_ul_handler  = FileUploadHandler(hs.clone());
    let resource_handler = StaticResourceHandler(hs.clone());
    let json_handler     = JSONHandler(hs.clone());
    let delete_handler   = DeleteHandler(hs.clone());
    let list_handler     = ListHandler(hs.clone());

    let router = RouterBuilder::new()
        .add(Route::get(r"(/|/index.html)").using(index_handler))
        .add(Route::post(r"(/|/index.html)").using(file_ul_handler))
        .add(Route::delete(r"/files/[^/]+$").using(delete_handler))
        .add(Route::get(r"/files/[^/]+$").using(file_dl_handler))
        .add(Route::get(r"/resource/[^/]+$").using(resource_handler))
        .add(Route::get(r"/json").using(json_handler))
        .add(Route::get(r"/list").using(list_handler))
        .set_handler_404(routes::handler_404)
        .set_handler_405(routes::handler_405)
        .set_handler_500(routes::handler_500)
        .build();

    let _ = srv.handle(router);
}


fn print_help(name: &str, opts: Options) {
    let brief = format!("Usage: {} [OPTIONS]", name);
    println!("{}", opts.usage(&brief));
}


fn print_version_info() {
    println!("na {}
Copyright (C) 2016 Srđan Panić <sp@srdja.me>.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.", VERSION);
}
