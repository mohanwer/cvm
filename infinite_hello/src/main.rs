use std::{process, time};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use sysinfo::System;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let running = Arc::new(AtomicUsize::new(0));
    let r = running.clone();
    ctrlc::set_handler(move || {
        let prev = r.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            println!("Exiting...");
        } else {
            process::exit(0);
        }
    })
        .expect("Error setting Ctrl-C handler");
    println!("Running...");
    let mut s = System::new_all();
    loop {
        s.refresh_all();
        let process = s.processes_by_name("infinite_hello".as_ref()).next().unwrap();
        println!("{:?} {}", process.name(), VERSION);
        sleep(Duration::new(5, 0));
        thread::sleep(time::Duration::from_secs(1));
    }
}
