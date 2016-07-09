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

use std::fs;
use std::io::Write;
use std::io::Read;
use std::fs::File;
use std::str;
use std::ops::Deref;
use url::percent_encoding::percent_decode;

use hyper::header::ContentDisposition;
use hyper::header::DispositionType;
use hyper::header::DispositionParam;
use hyper::header::Charset;
use hyper::header::ContentLength;
use hyper::header::Location;
use hyper::server::{Handler, Request, Response};
use hyper::status::StatusCode;
use hyper::{Get, Post};
use hyper::uri::RequestUri;

use directory::Directory;
use template;
use static_r::Resource;

use multipart::server::{Multipart, MultipartData};


pub struct RequestHandler {
    verbose:   bool,
    directory: Directory,
    resources: Resource,
}


impl RequestHandler {

    pub fn new(dir: Directory, res: Resource, verbose: bool) -> RequestHandler {
        RequestHandler {
            verbose:   verbose,
            directory: dir,
            resources: res,
        }
    }


    fn handle_get(&self, req: Request, mut res: Response) {
        let resources = self.directory.list_available_resources();

        let uri: String = match req.uri {
            RequestUri::AbsolutePath(path) => {
                percent_decode((&path).as_bytes()).decode_utf8().unwrap().deref().to_string()
            },
            RequestUri::AbsoluteUri(uri)   => uri.to_string(),
            _ => "fixme".to_string()
        };

        println_cond!(self.verbose, "Receiving a GET request from {} for {}",
                       req.remote_addr.to_string(), uri);

        if uri == "/" || uri == "/index.html" {
            let rendered = template::render(self.resources.r.get("/resource/index.html")
                                            .unwrap().to_string(), &resources);
            res.send(rendered.as_bytes()).unwrap();
            return;
        }
        if self.resources.r.contains_key(uri.as_str()) {
            res.send(self.resources.r.get(uri.as_str()).unwrap().as_bytes()).unwrap();
            return;
        }

        let mut name: Vec<u8> = Vec::new();
        name.extend_from_slice(uri[1..uri.len()].as_bytes());

        if resources.contains_key(&uri) {
            let r_name = resources.get(&uri).unwrap().clone(); // this should replace the if block
            let path = self.directory.full_path(r_name.name.clone());
            let meta = fs::metadata(&*path).unwrap();
            let mut file: File = File::open(&*path).unwrap();
            let len = meta.len() as usize;

            res.headers_mut().set(ContentLength(len as u64));

            res.headers_mut().set(ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![DispositionParam::Filename(
                    Charset::Ext("UTF-8".to_string()),
                    None,
                    name)]});

            let mut stream = res.start().unwrap();
            let mut buffer: [u8; 4096] = [0; 4096];
            let mut read_total: usize = 0;
            let mut sent_total: usize = 0;

            while read_total < len {
                let read: usize = file.read(&mut buffer).unwrap();
                let sent: usize = stream.write(&buffer[0 .. read]).unwrap();
                read_total = read_total + read;
                sent_total = sent_total + sent;
            }
            stream.end().unwrap();

            if sent_total != read_total {
                println!("");
            }
        }
    }

    fn handle_post(&self, req: Request, mut res: Response) {
        let remote_address = req.remote_addr.to_string();
        println_cond!(self.verbose, "Receiving a POST request from {}", remote_address);

        let multipart = Multipart::from_request(req).ok();
        if multipart.is_none() {
            println!("Err: Multipart missing!");
            return;
        }
        let mut mpu = multipart.unwrap();
        let multipart_field = mpu.read_entry();

        if multipart_field.is_err() {
            println!("Err: Multipart field missing");
            return;
        }
        let mp_data = multipart_field.unwrap();
        if mp_data.is_none() {
            println!("Err: Multipart data missing");
            return;
        }
        match mp_data.unwrap().data {
            MultipartData::File(mut file) => {
                let name = file.filename().unwrap().to_string();
                let path = self.directory.full_path(name);
                match file.save_as(path) {
                    Ok(f) => {
                        let p = f.path.to_str().unwrap();
                        println_cond!(self.verbose, "Written {} bytes to {}", f.size, p);
                        {
                            let stat: &mut StatusCode = res.status_mut();
                            *stat = StatusCode::Found;
                        }
                        res.headers_mut().set(Location("/".to_string()));
                        res.send(b"a").unwrap();
                        println_cond!(self.verbose, "Sending status code {}", StatusCode::Found.to_string());
                    },
                    Err(e) => {}
                }
            }
            MultipartData::Text(t) => {},
        }
    }
}


impl Handler for RequestHandler  {
    fn handle (&self, req: Request, res: Response) {
        match req.method {
            Post => {
                self.handle_post(req, res);
            },
            Get => {
                self.handle_get(req, res);
            },
            _ => return
        }
    }
}
