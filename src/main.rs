use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

mod errors;
mod http;

use bytes::{Bytes, BytesMut};
use errors::HttpError;

use crate::http::{Method, Request, Response, StatusCode};

const TMP_DIR: &str = "/tmp/data/codecrafters.io/http-server-tester";
const FILE_DIR: &str = "/files/";

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
    let mut buf = BytesMut::from(&[0; 1024][..]);
    loop {
        let n = stream.read(&mut buf).unwrap_or(0);

        if n == 0 {
            break;
        }

        let response = match Request::parse(&buf) {
            Ok(request) => match request.method {
                Method::Get => handle_get(request),
                Method::Put => todo!(),
                Method::Post => todo!(),
            },
            Err(e) => {
                eprintln!("{}", e);
                Ok(Response::not_found())
            }
        };

        if let Err(e) = stream.write_all(response.unwrap_or(Response::not_found()).as_ref()) {
            eprintln!("{e}");
        }
    }
}

fn handle_get(request: Request) -> Result<Bytes, HttpError> {
    let mut response = Response::default();
    let file_bytes: Vec<u8>;
    match request.path.to_str() {
        Some(str) if str.starts_with("/echo") => {
            let msg = request
                .path
                .file_name()
                .ok_or(HttpError::InvalidRequestFormat)?
                .to_str()
                .ok_or(HttpError::InvalidRequestFormat)?;
            response.status(StatusCode::Ok);
            response.content_type("text/plain");
            response.content_len(msg.len());
            response.body(msg);
        }
        Some(str) if str.starts_with("/user-agent") => {
            for line in request.headers {
                if line.to_lowercase().starts_with("user-agent:") {
                    let body = line
                        .split_once(':')
                        .ok_or(HttpError::InvalidRequestFormat)?
                        .1
                        .trim();
                    response.body(body);
                    response.status(StatusCode::Ok);
                    response.content_type("text/plain");
                    response.content_len(body.len());
                }
            }
        }
        Some("/") => response.status(StatusCode::Ok),
        Some(str) if str.starts_with(FILE_DIR) => {
            let uri = format!("{}/{}", TMP_DIR, &str[FILE_DIR.len()..]);
            file_bytes = std::fs::read(uri)?;
            let file_str = std::str::from_utf8(file_bytes.as_ref())?;
            response.status(StatusCode::Ok);
            response.content_type("application/octet-stream");
            response.content_len(file_bytes.len());
            response.body(file_str);
        }
        _ => response.status(StatusCode::NotFound),
    }
    Ok(response.build())
}
