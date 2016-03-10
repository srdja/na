extern crate libc;
extern crate getopts;
extern crate hyper;
extern crate mime;
extern crate regex;

mod ip;
mod directory;
mod request;
mod ui;
mod stream;

use hyper::server::{Handler, Server};
use directory::Directory;
use request::RequestHandler;

use std::path::PathBuf;
use std::env;


fn main() {
    let current_dir   = env::current_dir().unwrap();
    let address       = ip::get_local_addresses().unwrap();
    let addr_and_port = format!("{}:9000", address[0]);

    println!("Serving contents of {} at {}", current_dir.to_str().unwrap(), addr_and_port);

    let directory     = Directory::new(current_dir);
    let req_handler   = RequestHandler::new(directory, true);

    Server::http(&*addr_and_port).unwrap()
            .handle(req_handler).unwrap();
}
