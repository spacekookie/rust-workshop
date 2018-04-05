use std::fs::File;
use std::io::{Error as IoError, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{self, JoinHandle};
use std::sync::Arc;

fn main() {
    /* Load a user template or provide a default one */
    let mut t = String::new();
    let path = std::env::args().nth(1);
    create_template(&mut t, &path);

    /* Wrap the template into an Atomic Reference Counter */
    let template = Arc::new(t);


    let addr = "127.0.0.1:7200";
    println!("Binding to http://{}", addr);
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {

        /* We clone the Arc to move a copy of the reference into our thread closure */
        let template = Arc::clone(&template);
        thread::spawn(move || {

            /* Move closure to capture everything from the thread (aka template) */
            handle_connection(stream, move |s: &mut TcpStream| {
                let mut buffer = [0; 512];
                s.read(&mut buffer).unwrap();

                let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}", template);
                s.write(response.as_bytes()).unwrap();
                s.flush().unwrap();
            })
        });
    }
}

/// Construct a template either from file or from default.
///
/// Borrow one parameter mutably and one immutably.
fn create_template(template: &mut String, path: &Option<String>) {
    let default = "<!DOCTYPE html>
<html>
    <head>
        <title>🎉</title>
    </head>
    <body>
        <h1>Hello there stranger</h1>
    </body>
</html>";

    match path {
        Some(p) => {
            let mut f = File::open(p).expect("file not found");
            f.read_to_string(template).unwrap();
        }
        None => template.push_str(default),
    }
}

/// This function checks if an incoming request is valid
/// and calls the provided callback with the extracted stream
/// to act on.
fn handle_connection<F: 'static>(stream: Result<TcpStream, IoError>, fun: F)
where
    F: Fn(&mut TcpStream),
{
    match stream {
        Ok(mut s) => fun(&mut s),
        Err(e) => println!("Invalid connection: {}", e),
    }
}
