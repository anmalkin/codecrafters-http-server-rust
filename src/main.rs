use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

mod errors;
mod http;

use bytes::{Bytes, BytesMut};

use crate::http::*;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let mut handles = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let handle = std::thread::spawn(move || handle_connection(stream));
                handles.push(handle);
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = BytesMut::from(&[0; 512][..]);
    loop {
        let n = stream.read(&mut buf).unwrap();

        if n == 0 {
            break;
        }

        let request = match HttpRequest::parse(&buf) {
            Ok(request) => request,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        let mut response = HttpResponse::default();

        match request.method {
            HttpMethod::Get => {
                if !request.path.has_root() {
                    response.status(HttpStatusCode::_404_);
                }
                if let Some(path) = request.path.parent() {
                    match path.to_str().unwrap() {
                        "/echo" => {
                            let msg = request.path.file_stem().unwrap().to_str().unwrap();
                            response.body(msg);
                            response
                                .header(HttpResponseHeader::ContentType(Bytes::from("text/plain")));
                            response.header(HttpResponseHeader::ContentLength(Bytes::from(
                                msg.len().to_string(),
                            )));
                        }
                        _ => response.status(HttpStatusCode::_404_),
                    }
                }
            }
            _ => response.status(HttpStatusCode::_404_),
        }
        println!("Responding with: {:#?}", response.as_bytes());
        match stream.write_all(response.as_bytes().as_ref()) {
            Ok(_) => continue,
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}
