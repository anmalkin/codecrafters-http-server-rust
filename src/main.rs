use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

mod errors;
mod http;

use bytes::{Bytes, BytesMut};
use errors::HttpError;

use crate::http::{Method, Request, Response, StatusCode};

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

        let response = match Request::parse(&buf) {
            Ok(req) => match req.method {
                Method::Get => handle_get(req),
                Method::Put => todo!(),
                Method::Post => todo!(),
            },
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        match response {
            Ok(response) => match stream.write_all(response.as_ref()) {
                Ok(_) => continue,
                Err(e) => eprint!("{e}"),
            },
            Err(e) => eprint!("{e}"),
        }
    }
}

fn handle_get(request: Request) -> Result<Bytes, HttpError> {
    let mut response = Response::default();
    match request.path.to_str() {
        Some(str) if str.starts_with("/echo") => {
            let msg = request
                .path
                .file_name()
                .ok_or(HttpError::InvalidRequestFormat)?
                .to_str()
                .ok_or(HttpError::InvalidRequestFormat)?;
            response.body(msg);
            response.status(StatusCode::Ok)
        }
        Some(str) if str.starts_with("/user-agent") => {
            for line in request.headers {
                if line.to_lowercase().starts_with("user-agent:") {
                    response.body(
                        line.split_once(':')
                            .ok_or(HttpError::InvalidRequestFormat)?
                            .1
                            .trim(),
                    );
                    response.status(StatusCode::Ok)
                }
            }
        }
        Some("/") => response.status(StatusCode::Ok),
        _ => {}
    }
    Ok(response.build())
}
