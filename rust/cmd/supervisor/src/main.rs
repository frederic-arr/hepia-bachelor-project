fn main() {
    println!("Hello from supervisor!");
    loop {
        std::thread::park();
    }
}
