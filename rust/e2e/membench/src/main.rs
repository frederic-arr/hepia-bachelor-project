use std::env;
use std::io::{Read as _, Write as _};
use std::net::{SocketAddr, TcpStream};

fn send_size(stream: &mut TcpStream, size_mb: usize) -> std::io::Result<()> {
    stream.write_all(format!("{size_mb}\n").as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn send_stop(stream: &mut TcpStream) -> std::io::Result<()> {
    stream.write_all(b"STOP\n")?;
    stream.flush()?;

    let mut buf = [0; 4096];
    while stream.read(&mut buf)? > 0 {}

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let addr: SocketAddr = args[1].parse().unwrap();
    let n_start: usize = args[2].parse().unwrap();
    let mut stream = TcpStream::connect(addr).unwrap();

    let mut size_mb = n_start;
    let mb_bytes = 1024 * 1024;

    let sz = std::mem::size_of::<usize>();

    loop {
        let size_bytes = size_mb.checked_mul(mb_bytes).unwrap();
        let n = size_bytes.checked_div(sz).unwrap();

        let mut data = Vec::<usize>::new();
        data.reserve_exact(n);

        for (i, cell) in data.iter_mut().enumerate() {
            *cell = i;
        }

        let ok = data.iter().enumerate().all(|(i, cell)| *cell == i);
        if !ok {
            send_stop(&mut stream).unwrap();
            return;
        }

        send_size(&mut stream, size_mb).unwrap();
        drop(data);

        size_mb = size_mb.checked_add(1).unwrap();
    }
}
