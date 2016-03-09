/// Passed to server as handler

use std::fs;
use std::io::Write;
use std::io::Read;
use std::fs::File;
use std::str;

use mime::Attr;
use hyper::header::{ContentDisposition, DispositionType, ContentType,
                    DispositionParam, Charset, ContentLength, Location};

use hyper::server::{Handler, Request, Response};
use hyper::status::StatusCode;
use hyper::{Get, Post};

use directory::Directory;
use ui;
use stream;

///
///
///
pub struct RequestHandler {
    verbose:   bool,
    directory: Directory,
}



impl RequestHandler {


    pub fn new(dir: Directory, verbose: bool) -> RequestHandler {
        RequestHandler {
            verbose:   verbose,
            directory: dir,
        }
    }


    fn handle_get(&self, req: Request, mut res: Response) {
        let resources = self.directory.list_available_resources();
        let uri = req.uri.to_string();

        if self.verbose {
            println!("Receiving a GET request from {} for {}",
                     req.remote_addr.to_string(),
                     uri);
        }
        if uri == "/" {
            res.send(ui::render_ui(&resources).as_bytes()).unwrap();
            return;
        }

        let mut name: Vec<u8> = Vec::new();
        name.extend_from_slice(uri[1..uri.len()].as_bytes());

        if resources.contains(&uri) {
            let path = self.directory.full_path(uri);
            let meta = fs::metadata(&*path).unwrap();
            let mut file: File = File::open(&*path).unwrap();
            let len = meta.len() as usize;

            res.headers_mut().set(ContentLength(len as u64));

            res.headers_mut().set(ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![DispositionParam::Filename(
                    Charset::Iso_8859_1,
                    None,
                    name)]});

            let mut stream = res.start().unwrap();
            let mut buffer: [u8; 1024] = [0; 1024];
            let mut read_total: usize = 0;
            let mut sent_total: usize = 0;

            while read_total < len {
                let read: usize = file.read(&mut buffer).unwrap();
                let sent: usize = stream.write(&buffer[0 .. read]).unwrap();
                read_total = read_total + read;
                sent_total = sent_total + sent;
            }
            stream.end();

            if sent_total != read_total {
                println!("");
            }
        }
    }


    fn get_filename_from_form(&self, form: &str) -> Result<String, String> {
        let file_name = "filename=";
        let mut name  = "".to_string();

        for s in form.split(" ") {
            if s.contains(file_name) {
                let tmp: Vec<&str> = s.split("\"").collect();
                if tmp.len() != 2 && tmp[0] != file_name {
                    return Err("Error: Malformed form".to_string());
                }
                name = tmp[1].to_string();
            }
        }
        if name == "" {
            return Err("Error: File name not found".to_string());
        }
        Ok(name)
    }


    /// Return status code
    fn handle_post(&self, mut req: Request, mut res: Response) -> Result<usize, String> {
        let uri  = req.uri.to_string();
        let addr = req.remote_addr.to_string();

        if self.verbose {
            println!("Receiving a POST request from {}", addr);
        }
        if uri != "/" {
            return Err("Invalid request uri".to_string());
        }

        let adv = stream::advance_stream(&mut req, 500, "filename=\"".to_string(), true);
        println!("{}", adv.unwrap());

        let mut name_buffer: [u8; 200] = [0; 200];
        let result;
        {
            let mut name_writer = stream::WriteBuffer::new(&mut name_buffer);
            result = stream::write_stream(&mut req, &mut name_writer, 200, "\"".to_string(), false).unwrap();
        }
        println!("name is {}", result);
        // advance to filename with no write stream

        // headers.get_boundary()

       // let file_name = self.parse_post_form(&mut req).unwrap();

        // Borrow scope
        {
            let stat: &mut StatusCode = res.status_mut();
            *stat = StatusCode::Found;
        }

        res.headers_mut().set(Location("/".to_string()));
        res.send(b"Something").unwrap();
        println!("Sending status code {}", StatusCode::Found.to_string());

        Ok(0)
    }


    fn handle_requests(&self, req: Request, res: Response) {
        match req.method {
            Post => {
                match self.handle_post(req, res) {
                    Ok (n)   => return,
                    Err(err) => println!("Error: {:?}", err)
                }
            },
            Get => {
                self.handle_get(req, res);
            },
            _ => return
        }
    }
}


impl Handler for RequestHandler {
    fn handle (&self, req: Request, res: Response) {
        self.handle_requests(req, res);
    }
}
