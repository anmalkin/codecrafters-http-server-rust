use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

mod errors;
mod http;

use bytes::{Bytes, BytesMut};

use crate::http::{
    HttpMethod, HttpRequest, HttpResponse, HttpResponseHeader, HttpStatusCode, Serialize,
};

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

        if let HttpMethod::Get = request.method {
            match request.path.to_str() {
                Some(str) if str.starts_with("/echo") => {
                    let msg = request.path.file_stem().and_then(|p| p.to_str()).unwrap();
                    response.body(msg);
                    response.header(HttpResponseHeader::ContentType(Bytes::from("text/plain")));
                    response.header(HttpResponseHeader::ContentLength(Bytes::from(
                        msg.len().to_string(),
                    )));
                }
                Some(str) if str.starts_with("/user-agent") => todo!(),
                Some("/") => {response.status(HttpStatusCode::Ok)}
                _ => {}
            }
        }

        match stream.write_all(response.serialize().as_ref()) {
            Ok(_) => continue,
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}
