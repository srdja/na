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

use std::env;


fn main() {
    // root should be the directory from which the program is running to that it automatically
    // seves itself for download

    let current_dir = env::current_dir().unwrap();

    let address = ip::get_local_addresses().unwrap();
    let addr_and_port = format!("{}:9000", address[0]);

    let directory   = Directory::new(current_dir.to_str().unwrap().to_string());
    let req_handler = RequestHandler::new(directory, true);

    println!("Serving contents of {} at {}", current_dir.to_str().unwrap(), addr_and_port);

    Server::http(&*addr_and_port).unwrap()
            .handle(req_handler).unwrap();
}
