use std::{io::Write, net::{TcpListener, TcpStream}};

const STATUS_200: &[u8] = b"HTTP/1.1 200 OK\r\n\r\n";

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
    match stream.write_all(STATUS_200) {
        Ok(_) => {},
        Err(e) => println!("Error: {}", e),
    }
}
