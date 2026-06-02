fn main() {
    println!("Hello from Rust!");
    loop {
        std::thread::park();
    }
}
