use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, Error as IoError};

fn main() {
    // We could also read the address from the user via std::env::args()
    let addr = "127.0.0.1:7200";
    println!("Binding to http://{}", addr);
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        handle_connection(stream, |s: &mut TcpStream| {
            let mut buffer = [0; 512];
            s.read(&mut buffer).unwrap();

            let response = format!("HTTP/1.1 200 OK\r\n\r\nHello strangers!");
            s.write(response.as_bytes()).unwrap();
            s.flush().unwrap();
        });
    }
}

fn handle_connection<F: 'static>(stream: Result<TcpStream, IoError>, fun: F)
where
    F: Fn(&mut TcpStream),
{
    match stream {
        Ok(mut s) => fun(&mut s),
        Err(e) => println!("Invalid connection: {}", e),
    }
}
