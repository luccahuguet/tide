pub fn f() {
    std::thread::spawn(|| println!("ambient thread")).join().unwrap();
}
