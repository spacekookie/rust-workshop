//! A small example of how to write a (synchronous) multi-threaded webserver
#![allow(unused)]

use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::{fs::File, io::{Read, Write}};

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

/// An easy to use type that wraps around a static function
/// that is only called once, passed between threads.
///
/// This acts as the main Job type for our thread pool
type Job = Box<FnBox + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Execute something in this threadpool
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        Worker {
            id,
            thread: thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {} got a job; executing.", id);
                job.call_box();
            }),
        }
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
            f.read_to_string(template);
        }
        None => template.push_str(&format!("HTTP/1.1 200 OK\r\n\r\n{}", default)),
    }
}

fn main() {
    /* Shows how to borrow with and without mutability */
    let mut template = String::new();
    let path = std::env::args().nth(1);
    create_template(&mut template, &path);

    /* Setup the webserver & thread stack */
    let addr = "127.0.0.1:7200";
    let pool = ThreadPool::new(4);
    println!("Binding to http://{}", addr);
    let listener = TcpListener::bind(addr).unwrap();

    /* Handle incoming connetions */
    for stream in listener.incoming() {
        /* Demonstrate why this `clone` is necessary */
        let template = template.clone();
        pool.execute(move || {
            handle_connection(stream.unwrap(), &template);
        });
    }
}

/// Utility function that responds to GET requests with a template
fn handle_connection(mut stream: TcpStream, template: &String) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    /* Return a template. Here's one I made earlier */
    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", template);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
