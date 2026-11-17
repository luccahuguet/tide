//@ rules: CAP-1
// The closed table covers module aliases, missing inherent items and ambient sockets.
use std::env as environment;
use std::fs::{File, OpenOptions};
use std::time::SystemTime;

pub fn ambient() {
    let _ = environment::args(); //~ E0425 TD0501
    let _ = std::env::var("MODE"); //~ E0425 TD0501
    let _ = File::open("settings"); //~ E0599 TD0501
    let _ = OpenOptions::new().open("settings"); //~ E0599 TD0501
    let _ = SystemTime::now(); //~ E0599 TD0501
    let _ = std::io::stdin(); //~ E0425 TD0501
    let _ = std::net::TcpStream::connect("localhost:1"); //~ E0599 TD0501
    let _ = std::net::TcpListener::bind("localhost:1"); //~ E0599 TD0501
    let _ = thread_rng(); //~ E0425 TD0501
    std::thread::sleep(std::time::Duration::ZERO); //~ E0433 TD0501
    let _ = std::process::Command::new("tool"); //~ E0433 TD0501
    let _ = std::process::exit; //~ E0425 TD0501
}
