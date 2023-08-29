use std::{fs::Permissions, io::Write, net::TcpStream, thread::sleep, time::Duration};

fn main() {
    let connection = TcpStream::connect("127.0.0.1:9999");

    sleep(Duration::from_secs(2));
    match connection {
        Ok(mut stream) => {
            println!("Connected to the server!");
            stream.write(b"Hello from the client!");
            stream.flush();
            sleep(Duration::from_secs(2));
            stream.write(b"Hello from the client! 2");
            stream.flush();
        }
        Err(e) => {
            println!("Failed to connect: {e}");
        }
    }

    println!("Hello, world!");
}
