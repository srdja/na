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

use std::fs;
use std::io::Write;
use std::io::Read;
use std::fs::File;
use std::sync::Arc;
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
use hyper::uri::RequestUri;

use directory::Directory;
use template;
use static_r::Resource;

use multipart::server::{Multipart, MultipartData};


pub struct HandlerState {
    pub v: bool,
    pub d: Directory,
    pub r: Resource
}


pub struct FileDownloadHandler   (pub Arc<HandlerState>);
pub struct FileUploadHandler     (pub Arc<HandlerState>);
pub struct IndexHandler          (pub Arc<HandlerState>);
pub struct StaticResourceHandler (pub Arc<HandlerState>);
pub struct JSONHandler           (pub Arc<HandlerState>);


pub fn handler_400(mut res: Response) {
    {
        let stat: &mut StatusCode = res.status_mut();
        *stat = StatusCode::BadRequest;
    }
    let msg = "400 bad request";
    res.send(msg.as_bytes()).unwrap();
}


pub fn handler_404(_: Request, mut res: Response) {
    {
        let stat: &mut StatusCode = res.status_mut();
        *stat = StatusCode::NotFound;
    }
    let msg = "<html><head><meta charset=\"utf-8\"></head>\
               <body><pre>¯\\(º_o)/¯ 404 sorry, can't find that...</pre>\
               \n<a href=/><pre>Try going back</pre></a></body></html>";
    res.send(msg.as_bytes()).unwrap();
}


pub fn handler_500(_: Request, mut res: Response) {
    {
        let stat: &mut StatusCode = res.status_mut();
        *stat = StatusCode::InternalServerError;
    }
    let msg = "<html><head><meta charset=\"utf-8\"></head>\
               <body><pre>(╯°□°)╯︵ ┻━┻ 500 internal server error... \n \
               It's probably nothing, but then again, maybe the server is on fire!</pre>\
               \n<a href=/><pre>Try going back</pre></a></body></html>";
    res.send(msg.as_bytes()).unwrap();
}


impl Handler for IndexHandler {
    fn handle(&self, _: Request, res: Response) {
        let resource = self.0.d.list_available_resources();
        let rendered = template::render_html(self.0.r.r.get("/resource/index.html")
                                             .unwrap().to_string(), &resource);
        res.send(rendered.as_bytes()).unwrap();
    }
}


impl Handler for JSONHandler {
    fn handle(&self, _: Request, res: Response) {
        let resource = self.0.d.list_available_resources();
        let rendered = template::render_json(&resource);
        res.send(rendered.as_bytes()).unwrap();
    }
}


impl Handler for FileDownloadHandler {
    fn handle(&self, req: Request, mut res: Response) {
        let resources = self.0.d.list_available_resources();

        let uri: String = match req.uri {
            RequestUri::AbsolutePath(ref path) => {
                percent_decode((&path).as_bytes()).decode_utf8().unwrap().deref().to_string()
            },
            _ => {
                handler_404(req, res);
                return;
            }
        };
        println_cond!(self.0.v, "Receiving a GET request from {} for {}",
                       req.remote_addr.to_string(), uri);

        let segments: Vec<&str> = uri.split("/").collect();
        let str_name = segments.last().unwrap().to_string();
        let mut name: Vec<u8> = Vec::new();
        name.extend_from_slice(str_name.as_bytes());

        if !resources.contains_key(&str_name) {
            handler_404(req, res);
            return;
        }

        let r_name = resources.get(&str_name).unwrap().clone();
        let path = self.0.d.full_path(r_name.name.clone());
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
        let mut buffer: [u8; 8192] = [0; 8192];
        let mut read_total: usize = 0;
        let mut sent_total: usize = 0;

        while read_total < len {
            let read: usize = match file.read(&mut buffer) {
                Ok (b) => b,
                Err(e) => {
                    println_cond!(self.0.v,
                                  "Error: Unexpected end of stream while reading {}, \
                                   {} bytes read out of {}. [{}]",
                                  path.as_path().to_str().unwrap(), read_total, len, e);
                    return;
                }
            };
            let sent: usize = match stream.write(&buffer[0 .. read]) {
                Ok (b) => b,
                Err(e) => {
                    println_cond!(self.0.v,
                                  "Error: Unexpected end of stream while sending {}, \
                                   {} bytes sent out of {}. [{}]",
                                  path.as_path().to_str().unwrap(), sent_total, len, e);
                    return;
                }
            };
            read_total = read_total + read;
            sent_total = sent_total + sent;
        }
        stream.end().unwrap();

        println_cond!(self.0.v, "Sent a total of {} out of {} bytes to {} for request {}",
                      sent_total, len, req.remote_addr.to_string(), uri);
    }
}


impl Handler for FileUploadHandler {
    fn handle(&self, req: Request, mut res: Response) {
        let remote_address = req.remote_addr.to_string();
        println_cond!(self.0.v, "Receiving a POST request from {}", remote_address);

        let multipart = Multipart::from_request(req).ok();
        if multipart.is_none() {
            println_cond!(self.0.v, "Error: Bad POST request from {}. Multipart missing!",
                          remote_address);
            handler_400(res);
            return;
        }
        let mut mpu = multipart.unwrap();
        let multipart_field = mpu.read_entry();

        if multipart_field.is_err() {
            println_cond!(self.0.v, "Error: Bad POST request from {}. Multipart field missing!",
                          remote_address);
            handler_400(res);
            return;
        }
        let mp_data = multipart_field.unwrap();
        if mp_data.is_none() {
            println_cond!(self.0.v, "Error: Bad POST request from {}. Multipart data missing!",
                          remote_address);
            handler_400(res);
            return;
        }
        match mp_data.unwrap().data {
            MultipartData::File(mut file) => {
                let name = self.0.d.get_available_name(file.filename().unwrap().to_string());
                let path = self.0.d.full_path(name.clone());
                match file.save_as(path) {
                    Ok(f) => {
                        let p = f.path.to_str().unwrap();
                        println_cond!(self.0.v, "Written {} bytes to {}", f.size, p);
                        {
                            let stat: &mut StatusCode = res.status_mut();
                            *stat = StatusCode::Found;
                        }
                        res.headers_mut().set(Location("/".to_string()));
                        res.send(b"a").unwrap();
                        println_cond!(self.0.v, "Sending status code {}", StatusCode::Found.to_string());
                    },
                    Err(e) => {
                        println_cond!(self.0.v, "Error: Couldn't save {} to disk! \
                                                 {}", name, e);
                    }
                }
            }
            MultipartData::Text(_) => {},
        }
    }
}


impl Handler for StaticResourceHandler {
    fn handle(&self, req: Request, res: Response) {
        let uri: String = match req.uri {
            RequestUri::AbsolutePath(path) => {
                percent_decode((&path).as_bytes()).decode_utf8().unwrap().deref().to_string()
            },
            RequestUri::AbsoluteUri(uri) => uri.to_string(),
            _ => "fixme".to_string()
        };
        if self.0.r.r.contains_key(uri.as_str()) {
            res.send(self.0.r.r.get(uri.as_str()).unwrap().as_bytes()).unwrap();
            return;
        }
    }
}
