use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

const STATUS_200: &[u8] = b"HTTP/1.1 200 OK\r\n\r\n";
const STATUS_404: &[u8] = b"HTTP/1.1 404 Not Found\r\n\r\n";

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
                println!("error: {}", e);
            }
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf: Vec<u8> = vec![0; 512];
    loop {
        let n = stream.read(&mut buf).unwrap();

        if n == 0 {
            break;
        }

        // TODO: Better error handling
        let mut str = std::str::from_utf8(&buf[..]).unwrap().lines();
        let mut start_line = str.next().unwrap().split_whitespace();
        let method = start_line.next().unwrap();
        println!("HTTP method: {}", method);
        let target = start_line.next().unwrap();
        println!("Request target: {}", target);
        match target {
            "/" => match stream.write(STATUS_200) {
                Ok(_) => {}
                Err(e) => println!("Error: {}", e),
            },
            _ => match stream.write(STATUS_404) {
                Ok(_) => {}
                Err(e) => println!("Error: {}", e),
            },
        }
    }

    match stream.write_all(STATUS_200) {
        Ok(_) => {}
        Err(e) => println!("Error: {}", e),
    }
}
