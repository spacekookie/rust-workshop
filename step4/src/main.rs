use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::{fs::File, io::{Read, Write, Error as IoError}};


fn main() {
    /* Load a user template or provide a default one */
    let mut t = String::new();
    let path = std::env::args().nth(1);
    create_template(&mut t, path);

    /* Wrap the template into an Atomic Reference Counter */
    let template = Arc::new(t);

    /* Setup the webserver & thread stack */
    let addr = "127.0.0.1:7200";
    let pool = ThreadPool::new(4);
    println!("Binding to http://{}", addr);
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let template = Arc::clone(&template);

        /* This invokes ✨ MAGIC ✨ */
        pool.execute(move || {
            handle_connection(stream, move |s| {
                let mut buffer = [0; 512];
                s.read(&mut buffer).unwrap();

                let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}", template);
                s.write(response.as_bytes()).unwrap();
                s.flush().unwrap();
            });
        });
    }
}

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

    // By putting a "_" here it tells the Rust linter "don't worry 
    // about it being unused". That's pretty handy to get rid of 
    // warnings that you explicitly know not to be true. 
    // 
    // For example, here we only ever create "workers" and save things
    // into it. Because we never access it, the linter thinks it's
    // unused and throws a warning. Because clearly, we could just do all
    // the work in the constructor, right?
    // 
    // It doesn't understand the concept of keeping a reference to 
    // prevent the workers from being destroyed.
    _workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let mut _workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            _workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { _workers, sender }
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
    _id: usize,
    _thread: JoinHandle<()>,
}

impl Worker {
    fn new(_id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        Worker {
            _id,
            _thread: thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {} got a job; executing.", _id);
                job.call_box();
            }),
        }
    }
}

/// Construct a template either from file or from default.
///
/// Borrow one parameter mutably and one immutably.
fn create_template(template: &mut String, path: Option<String>) {
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
