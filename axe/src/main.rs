//! That's a nice webserver you have there.
//!
//! Wouldn't it be a shame if someone were to DDOS it?

extern crate reqwest;

fn main() {
    let addr = match std::env::args().nth(1) {
        Some(a) => a,
        None => {
            println!("Usage: hammer <addr>");
            std::process::exit(2)
        }
    };

    loop {
        let a = reqwest::get(&addr).unwrap().text().unwrap();
        println!("Body: {}", a[0..8]);
    }
}
